use ahash::{AHashMap, AHashSet};
use scale_value::Composite;
use serde_json::json;
use subxt::{
    OnlineClient, PolkadotConfig,
    config::RpcConfigFor,
    error::{BackendError, OnlineClientAtBlockError, RpcError},
    events::{Event, Events},
    rpcs::methods::legacy::LegacyRpcMethods,
};

use crate::{
    errors::{IndexError, internal_error},
    protocol::{DecodedEvent, EventBlockProof, EventRef, HexBytes},
};

pub(crate) struct FetchedBlock {
    pub(crate) block_number: u32,
    pub(crate) spec_version: u32,
    pub(crate) timestamp: u64,
    pub(crate) events: Events<PolkadotConfig>,
}

pub(crate) async fn fetch_block_events(
    api: &OnlineClient<PolkadotConfig>,
    rpc: &LegacyRpcMethods<RpcConfigFor<PolkadotConfig>>,
    block_number: u32,
) -> Result<FetchedBlock, IndexError> {
    let block_hash = rpc
        .chain_get_block_hash(Some(block_number.into()))
        .await?
        .ok_or(IndexError::BlockNotFound(block_number))?;

    let at_block = api.at_block(block_hash).await.map_err(|err| {
        if is_state_pruned_error(&err) {
            IndexError::StatePruningMisconfigured { block_number }
        } else {
            err.into()
        }
    })?;
    let spec_version = at_block.spec_version();
    let timestamp = fetch_block_timestamp(rpc, block_hash, block_number).await?;
    let events = at_block.events().fetch().await?;

    Ok(FetchedBlock {
        block_number,
        spec_version,
        timestamp,
        events,
    })
}

pub(crate) async fn hydrate_event_refs(
    api: &OnlineClient<PolkadotConfig>,
    rpc: &LegacyRpcMethods<RpcConfigFor<PolkadotConfig>>,
    event_refs: &[EventRef],
) -> Result<Vec<DecodedEvent>, IndexError> {
    if event_refs.is_empty() {
        return Ok(Vec::new());
    }

    let mut requested_by_block = AHashMap::<u32, AHashSet<u32>>::new();
    for event_ref in event_refs {
        requested_by_block
            .entry(event_ref.block_number)
            .or_default()
            .insert(event_ref.event_index);
    }

    let mut decoded_by_ref = AHashMap::<(u32, u32), DecodedEvent>::new();
    for block_number in requested_by_block.keys().copied() {
        let fetched = fetch_block_events(api, rpc, block_number).await?;
        decode_requested_block_events(
            &fetched,
            requested_by_block.get(&block_number).unwrap(),
            &mut decoded_by_ref,
        )?;
    }

    let mut decoded_events = Vec::with_capacity(event_refs.len());
    for event_ref in event_refs {
        let decoded_event = decoded_by_ref
            .remove(&(event_ref.block_number, event_ref.event_index))
            .ok_or_else(|| {
                internal_error(format!(
                    "event not found on node for block {} event {}",
                    event_ref.block_number, event_ref.event_index
                ))
            })?;
        decoded_events.push(decoded_event);
    }

    Ok(decoded_events)
}

pub(crate) async fn fetch_event_block_proofs(
    rpc: &LegacyRpcMethods<RpcConfigFor<PolkadotConfig>>,
    event_refs: &[EventRef],
) -> Result<Vec<EventBlockProof>, IndexError> {
    if event_refs.is_empty() {
        return Ok(Vec::new());
    }

    let mut block_numbers = AHashSet::<u32>::new();
    for event_ref in event_refs {
        block_numbers.insert(event_ref.block_number);
    }

    let mut proofs = Vec::with_capacity(block_numbers.len());
    for block_number in block_numbers {
        proofs.push(fetch_event_block_proof(rpc, block_number).await?);
    }
    proofs.sort_by_key(|proof| std::cmp::Reverse(proof.block_number));

    Ok(proofs)
}

async fn fetch_event_block_proof(
    rpc: &LegacyRpcMethods<RpcConfigFor<PolkadotConfig>>,
    block_number: u32,
) -> Result<EventBlockProof, IndexError> {
    let storage_key = system_events_storage_key();
    let block_hash = rpc
        .chain_get_block_hash(Some(block_number.into()))
        .await?
        .ok_or(IndexError::BlockNotFound(block_number))?;
    let header = rpc
        .chain_get_header(Some(block_hash))
        .await?
        .ok_or(IndexError::BlockNotFound(block_number))?;
    let storage_value = rpc
        .state_get_storage(&storage_key, Some(block_hash))
        .await?
        .ok_or_else(|| internal_error(format!("System.Events missing at block {block_number}")))?;
    let read_proof = rpc
        .state_get_read_proof([storage_key.as_slice()], Some(block_hash))
        .await?;

    Ok(EventBlockProof {
        block_number,
        block_hash: HexBytes(block_hash.as_ref().to_vec()),
        header: serde_json::to_value(&header)?,
        storage_key: HexBytes(storage_key.to_vec()),
        storage_value: HexBytes(storage_value),
        storage_proof: read_proof
            .proof
            .into_iter()
            .map(|bytes| HexBytes(bytes.0))
            .collect(),
    })
}

pub(crate) fn system_events_storage_key() -> [u8; 32] {
    storage_key(b"System", b"Events")
}

fn timestamp_now_storage_key() -> [u8; 32] {
    storage_key(b"Timestamp", b"Now")
}

fn storage_key(pallet: &[u8], entry: &[u8]) -> [u8; 32] {
    let pallet_hash = sp_crypto_hashing::twox_128(pallet);
    let entry_hash = sp_crypto_hashing::twox_128(entry);
    let mut res = [0u8; 32];
    res[..16].copy_from_slice(&pallet_hash);
    res[16..].copy_from_slice(&entry_hash);
    res
}

async fn fetch_block_timestamp(
    rpc: &LegacyRpcMethods<RpcConfigFor<PolkadotConfig>>,
    block_hash: subxt::utils::H256,
    block_number: u32,
) -> Result<u64, IndexError> {
    let storage_key = timestamp_now_storage_key();
    let Some(storage_value) = rpc
        .state_get_storage(&storage_key, Some(block_hash))
        .await?
    else {
        return Ok(0);
    };

    decode_timestamp_storage_value(&storage_value).ok_or_else(|| {
        internal_error(format!(
            "invalid Timestamp::Now value at block {block_number}: expected SCALE-encoded u64"
        ))
    })
}

fn decode_timestamp_storage_value(bytes: &[u8]) -> Option<u64> {
    match bytes.len() {
        8.. => Some(u64::from_le_bytes(bytes[..8].try_into().ok()?)),
        4 => Some(u32::from_le_bytes(bytes.try_into().ok()?) as u64),
        _ => None,
    }
}

fn decode_requested_block_events(
    fetched: &FetchedBlock,
    requested_indexes: &AHashSet<u32>,
    decoded_by_ref: &mut AHashMap<(u32, u32), DecodedEvent>,
) -> Result<(), IndexError> {
    if requested_indexes.is_empty() {
        return Ok(());
    }

    for event_result in fetched.events.iter() {
        let event = event_result?;
        let event_index = event.index();
        if !requested_indexes.contains(&event_index) {
            continue;
        }

        let decoded_event = decode_event_details(
            fetched.block_number,
            fetched.spec_version,
            fetched.timestamp,
            &event,
        )?;
        decoded_by_ref.insert((fetched.block_number, event_index), decoded_event);
    }

    Ok(())
}

fn decode_event_details(
    block_number: u32,
    spec_version: u32,
    timestamp: u64,
    event: &Event<PolkadotConfig>,
) -> Result<DecodedEvent, IndexError> {
    let event_index = event.index();
    let fields: Composite<()> = event.decode_fields_unchecked_as::<Composite<()>>()?;

    Ok(DecodedEvent {
        block_number,
        event_index,
        timestamp,
        event: encode_event_value(
            spec_version,
            event.pallet_name(),
            event.event_name(),
            event.pallet_index(),
            event.event_index(),
            event_index,
            &fields,
        ),
    })
}

pub(crate) fn encode_event_value(
    spec_version: u32,
    pallet_name: &str,
    event_name: &str,
    pallet_index: u8,
    variant_index: u8,
    event_index: u32,
    fields: &Composite<()>,
) -> serde_json::Value {
    json!({
        "specVersion": spec_version,
        "palletName": pallet_name,
        "eventName": event_name,
        "palletIndex": pallet_index,
        "variantIndex": variant_index,
        "eventIndex": event_index,
        "fields": crate::indexer::composite_to_json(fields),
    })
}

fn is_state_pruned_error(err: &OnlineClientAtBlockError) -> bool {
    match err {
        OnlineClientAtBlockError::CannotGetSpecVersion {
            reason: BackendError::Rpc(RpcError::ClientError(subxt::rpcs::Error::User(user_err))),
            ..
        } => user_err.code == 4003 && user_err.message.contains("State already discarded"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale_value::{Primitive, Value, ValueDef};

    #[test]
    fn encode_event_value_preserves_existing_wire_shape() {
        let fields = Composite::Named(vec![(
            "amount".into(),
            Value {
                value: ValueDef::Primitive(Primitive::U128(999)),
                context: (),
            },
        )]);

        assert_eq!(
            encode_event_value(1234, "Balances", "Deposit", 5, 2, 7, &fields),
            json!({
                "specVersion": 1234,
                "palletName": "Balances",
                "eventName": "Deposit",
                "palletIndex": 5,
                "variantIndex": 2,
                "eventIndex": 7,
                "fields": {"amount": "999"},
            })
        );
    }

    #[test]
    fn decode_timestamp_storage_value_reads_scale_u64() {
        assert_eq!(
            decode_timestamp_storage_value(&1234u64.to_le_bytes()),
            Some(1234)
        );
    }

    #[test]
    fn system_events_storage_key_matches_substrate_layout() {
        assert_eq!(
            hex::encode(system_events_storage_key()),
            "26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7"
        );
    }

    #[test]
    fn timestamp_now_storage_key_matches_substrate_layout() {
        assert_eq!(
            hex::encode(timestamp_now_storage_key()),
            "f0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb"
        );
    }
}
