use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    error::Error,
    fs, io,
    net::TcpListener,
    path::{Path, PathBuf},
    process,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use subxt::{OnlineClient, PolkadotConfig};
use tokio::time::sleep;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryExpectation {
    pub description: String,
    pub key: Value,
    pub min_events: usize,
    pub expected_event_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedManifest {
    pub mode: String,
    pub genesis_hash: String,
    pub start_block: u32,
    pub end_block: u32,
    pub total_blocks: u32,
    pub transactions_submitted: u32,
    pub synthetic_event_count: u32,
    pub queries: Vec<QueryExpectation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkReport {
    pub chain_tip: u32,
    pub indexed_blocks: u32,
    pub queue_depth: u8,
    pub synthetic_event_count: u32,
    pub elapsed_seconds: f64,
    pub blocks_per_second: f64,
    pub synthetic_events_per_second: f64,
}

pub fn key_u32(name: &str, value: u32) -> Value {
    json!({
        "type": "Custom",
        "value": {
            "name": name,
            "kind": "u32",
            "value": value,
        }
    })
}

pub fn key_bytes32(name: &str, value: [u8; 32]) -> Value {
    json!({
        "type": "Custom",
        "value": {
            "name": name,
            "kind": "bytes32",
            "value": bytes32_hex(value),
        }
    })
}

pub fn key_account(account_id: [u8; 32]) -> Value {
    key_bytes32("account_id", account_id)
}

pub fn bytes32_hex(value: [u8; 32]) -> String {
    format!("0x{}", hex::encode(value))
}

pub fn synthetic_digest(batch_id: u32, seq: u32) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[..4].copy_from_slice(&batch_id.to_be_bytes());
    bytes[4..8].copy_from_slice(&seq.to_be_bytes());
    for (idx, byte) in bytes[8..].iter_mut().enumerate() {
        *byte = batch_id
            .wrapping_mul(31)
            .wrapping_add(seq.wrapping_mul(17))
            .wrapping_add(idx as u32) as u8;
    }
    bytes
}

pub async fn fetch_genesis_hash(url: &str) -> Result<String, Box<dyn Error>> {
    let api = OnlineClient::<PolkadotConfig>::from_insecure_url(url).await?;
    Ok(hex::encode(api.genesis_hash().as_ref()))
}

pub async fn current_block_number(url: &str) -> Result<u32, Box<dyn Error>> {
    let api = OnlineClient::<PolkadotConfig>::from_insecure_url(url).await?;
    let at_block = api.at_current_block().await?;
    Ok(at_block
        .block_number()
        .try_into()
        .map_err(|_| io::Error::other("block number exceeds u32"))?)
}

pub async fn wait_for_node(
    url: &str,
    min_block: u32,
    timeout: Duration,
) -> Result<u32, Box<dyn Error>> {
    let deadline = Instant::now() + timeout;
    loop {
        match current_block_number(url).await {
            Ok(block) if block >= min_block => return Ok(block),
            Ok(_) | Err(_) if Instant::now() < deadline => sleep(Duration::from_millis(200)).await,
            Ok(block) => {
                return Err(io::Error::other(format!(
                    "node did not reach block {min_block}; last block {block}"
                ))
                .into());
            }
            Err(err) => return Err(err),
        }
    }
}

pub fn render_synthetic_index_spec(
    url: &str,
    genesis_hash: &str,
) -> Result<String, Box<dyn Error>> {
    Ok(format!(
        concat!(
            "name = \"synthetic-runtime\"\n",
            "genesis_hash = \"{}\"\n",
            "default_url = \"{}\"\n",
            "spec_change_blocks = [0]\n\n",
            "[keys]\n",
            "account_id = \"bytes32\"\n",
            "record_id = \"u32\"\n",
            "topic = \"u32\"\n",
            "digest = \"bytes32\"\n",
            "batch_id = \"u32\"\n",
            "seq = \"u32\"\n\n",
            "[[pallets]]\n",
            "name = \"Balances\"\n",
            "events = [\n",
            "  {{ name = \"Transfer\", params = [\n",
            "    {{ field = \"from\", key = \"account_id\" }},\n",
            "    {{ field = \"to\", key = \"account_id\" }},\n",
            "  ] }},\n",
            "]\n\n",
            "[[pallets]]\n",
            "name = \"Synthetic\"\n",
            "events = [\n",
            "  {{ name = \"RecordStored\", params = [\n",
            "    {{ field = \"record_id\", key = \"record_id\" }},\n",
            "    {{ field = \"owner\", key = \"account_id\" }},\n",
            "    {{ field = \"digest\", key = \"digest\" }},\n",
            "    {{ field = \"topics\", key = \"topic\", multi = true }},\n",
            "  ] }},\n",
            "  {{ name = \"LinksStored\", params = [\n",
            "    {{ field = \"record_id\", key = \"record_id\" }},\n",
            "    {{ field = \"related_ids\", key = \"record_id\", multi = true }},\n",
            "    {{ field = \"related_digests\", key = \"digest\", multi = true }},\n",
            "  ] }},\n",
            "  {{ name = \"BurstEmitted\", params = [\n",
            "    {{ field = \"batch_id\", key = \"batch_id\" }},\n",
            "    {{ field = \"seq\", key = \"seq\" }},\n",
            "    {{ field = \"owner\", key = \"account_id\" }},\n",
            "    {{ field = \"digest\", key = \"digest\" }},\n",
            "  ] }},\n",
            "]\n"
        ),
        genesis_hash, url
    ))
}

pub fn write_synthetic_index_spec(
    path: &Path,
    url: &str,
    genesis_hash: &str,
) -> Result<(), Box<dyn Error>> {
    let rendered = render_synthetic_index_spec(url, genesis_hash)?;
    fs::write(path, rendered)?;
    Ok(())
}

pub fn unique_temp_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", process::id()))
}

pub fn pick_unused_port() -> io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn recv_json_ws(socket: &mut WsStream) -> Result<Value, Box<dyn Error>> {
    while let Some(message) = socket.next().await {
        match message? {
            Message::Text(text) => return Ok(serde_json::from_str(text.as_ref())?),
            Message::Binary(bytes) => return Ok(serde_json::from_slice(&bytes)?),
            Message::Ping(payload) => socket.send(Message::Pong(payload)).await?,
            Message::Close(frame) => {
                return Err(io::Error::other(format!("websocket closed: {frame:?}")).into());
            }
            _ => continue,
        }
    }

    Err(io::Error::other("websocket ended before a response was received").into())
}

pub struct JsonWsClient {
    socket: WsStream,
    next_request_id: u64,
}

impl JsonWsClient {
    pub async fn connect(url: &str) -> Result<Self, Box<dyn Error>> {
        let (socket, _) = connect_async(url).await?;
        Ok(Self {
            socket,
            next_request_id: 1,
        })
    }

    async fn send_json(&mut self, request: Value) -> Result<(), Box<dyn Error>> {
        self.socket
            .send(Message::Text(request.to_string().into()))
            .await?;
        Ok(())
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value, Box<dyn Error>> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        let request = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params,
        });

        self.send_json(request).await?;
        loop {
            let message = recv_json_ws(&mut self.socket).await?;
            if message.get("id").and_then(|id| id.as_u64()) == Some(request_id) {
                return Ok(message);
            }
        }
    }

    pub async fn get_events(&mut self, key: Value, limit: u16) -> Result<Value, Box<dyn Error>> {
        self.request(
            "acuity_getEvents",
            json!({
                "key": key,
                "limit": limit,
            }),
        )
        .await
    }
}

pub async fn request_json_ws(url: &str, request: Value) -> Result<Value, Box<dyn Error>> {
    let (mut socket, _) = connect_async(url).await?;
    socket
        .send(Message::Text(request.to_string().into()))
        .await?;
    recv_json_ws(&mut socket).await
}

pub async fn fetch_status(indexer_url: &str) -> Result<Value, Box<dyn Error>> {
    request_json_ws(
        indexer_url,
        json!({"jsonrpc": "2.0", "id": 1, "method": "acuity_indexStatus"}),
    )
    .await
}

pub fn spans_cover_tip(status_response: &Value, expected_tip: u32) -> bool {
    status_response["result"]["spans"]
        .as_array()
        .is_some_and(|spans| {
            spans.iter().any(|span| {
                span["start"].as_u64().unwrap_or(u64::MAX) <= 1
                    && span["end"].as_u64().unwrap_or(0) >= u64::from(expected_tip)
            })
        })
}

pub async fn wait_for_indexed_tip(
    indexer_url: &str,
    expected_tip: u32,
    timeout: Duration,
) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + timeout;
    loop {
        match fetch_status(indexer_url).await {
            Ok(status) if spans_cover_tip(&status, expected_tip) => return Ok(()),
            Ok(_) | Err(_) if Instant::now() < deadline => sleep(Duration::from_millis(200)).await,
            Ok(status) => {
                return Err(io::Error::other(format!(
                    "indexer did not reach tip {expected_tip}: {status}"
                ))
                .into());
            }
            Err(err) => return Err(err),
        }
    }
}

pub async fn get_events(
    indexer_url: &str,
    key: Value,
    limit: u16,
) -> Result<Value, Box<dyn Error>> {
    request_json_ws(
        indexer_url,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "acuity_getEvents",
            "params": {
                "key": key,
                "limit": limit,
            },
        }),
    )
    .await
}

pub fn events_len(response: &Value) -> usize {
    response["result"]["events"]
        .as_array()
        .map(Vec::len)
        .unwrap_or_default()
}

pub fn decoded_event_names(response: &Value) -> Vec<String> {
    response["result"]["events"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|event| event["event"]["eventName"].as_str().map(ToOwned::to_owned))
        .collect()
}

pub fn validate_query_expectation(
    query: &QueryExpectation,
    response: &Value,
) -> Result<(), String> {
    let count = events_len(response);
    if count < query.min_events {
        return Err(format!(
            "query '{}' returned {count} events, expected at least {}",
            query.description, query.min_events
        ));
    }

    let names = decoded_event_names(response);
    for expected in &query.expected_event_names {
        if !names.iter().any(|name| name == expected) {
            return Err(format!(
                "query '{}' missing decoded event '{}' in {:?}",
                query.description, expected, names
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Key builders ──────────────────────────────────────────────────────

    #[test]
    fn key_u32_builds_custom_key_json() {
        let key = key_u32("batch_id", 77);
        assert_eq!(
            key,
            json!({"type": "Custom", "value": {"name": "batch_id", "kind": "u32", "value": 77}})
        );
    }

    #[test]
    fn key_bytes32_builds_custom_key_json_with_hex_value() {
        let key = key_bytes32("digest", [0xAB; 32]);
        assert_eq!(key["type"], "Custom");
        assert_eq!(key["value"]["name"], "digest");
        assert_eq!(key["value"]["kind"], "bytes32");
        assert_eq!(
            key["value"]["value"],
            json!(format!("0x{}", hex::encode([0xAB; 32])))
        );
    }

    #[test]
    fn key_account_uses_account_id_name_bytes32_kind() {
        let key = key_account([0xCD; 32]);
        assert_eq!(key["value"]["name"], "account_id");
        assert_eq!(key["value"]["kind"], "bytes32");
    }

    #[test]
    fn bytes32_hex_prefixes_encoded_bytes() {
        assert_eq!(bytes32_hex([0u8; 32]), format!("0x{}", "00".repeat(32)));
        assert_eq!(bytes32_hex([0xFF; 32]), format!("0x{}", "ff".repeat(32)));
    }

    // ─── synthetic_digest ─────────────────────────────────────────────────

    #[test]
    fn synthetic_digest_encodes_batch_and_seq_in_first_eight_bytes() {
        let digest = synthetic_digest(0x01020304, 0x05060708);
        assert_eq!(&digest[..4], &[1, 2, 3, 4]);
        assert_eq!(&digest[4..8], &[5, 6, 7, 8]);
    }

    #[test]
    fn synthetic_digest_is_deterministic_for_same_inputs() {
        assert_eq!(synthetic_digest(123, 456), synthetic_digest(123, 456));
        assert_ne!(synthetic_digest(123, 456), synthetic_digest(123, 457));
    }

    // ─── Index spec rendering ──────────────────────────────────────────────

    #[test]
    fn render_synthetic_index_spec_embeds_genesis_hash_and_url() {
        let rendered = render_synthetic_index_spec("ws://127.0.0.1:9944", "deadbeef").unwrap();
        assert!(rendered.contains("genesis_hash = \"deadbeef\""));
        assert!(rendered.contains("default_url = \"ws://127.0.0.1:9944\""));
        assert!(rendered.contains("spec_change_blocks = [0]"));
    }

    #[test]
    fn render_synthetic_index_spec_parses_as_valid_toml() {
        let rendered = render_synthetic_index_spec("ws://127.0.0.1:9944", "abcdef").unwrap();
        let doc: toml::Value = toml::from_str(&rendered).unwrap();
        assert_eq!(doc["name"].as_str(), Some("synthetic-runtime"));
        assert_eq!(doc["genesis_hash"].as_str(), Some("abcdef"));
        assert!(doc.get("keys").and_then(toml::Value::as_table).is_some());
        let pallets = doc["pallets"].as_array().unwrap();
        assert_eq!(pallets.len(), 2);
    }

    #[test]
    fn write_synthetic_index_spec_round_trips_to_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("synthetic.toml");
        write_synthetic_index_spec(&path, "ws://127.0.0.1:9944", "beef").unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("genesis_hash = \"beef\""));
    }

    // ─── Status / events helpers ───────────────────────────────────────────

    #[test]
    fn spans_cover_tip_true_when_a_span_reaches_tip() {
        let status = json!({"result": {"spans": [{"start": 1, "end": 500}]}});
        assert!(spans_cover_tip(&status, 500));
        assert!(spans_cover_tip(&status, 100));
    }

    #[test]
    fn spans_cover_tip_false_before_tip() {
        let status = json!({"result": {"spans": [{"start": 1, "end": 400}]}});
        assert!(!spans_cover_tip(&status, 500));
    }

    #[test]
    fn spans_cover_tip_empty_or_missing_spans_is_false() {
        assert!(!spans_cover_tip(&json!({}), 1));
        assert!(!spans_cover_tip(&json!({"result": {"spans": []}}), 1));
    }

    #[test]
    fn events_len_counts_result_events_and_defaults_zero() {
        let response = json!({"result": {"events": [{}, {}, {}]}});
        assert_eq!(events_len(&response), 3);
        assert_eq!(events_len(&json!({"result": {}})), 0);
        assert_eq!(events_len(&json!({})), 0);
    }

    #[test]
    fn decoded_event_names_extracts_names_in_order() {
        let response = json!({"result": {"events": [
            {"event": {"eventName": "A"}},
            {"event": {"eventName": "B"}},
            {"event": {}},
        ]}});
        assert_eq!(
            decoded_event_names(&response),
            vec!["A".to_string(), "B".to_string()]
        );
    }

    #[test]
    fn validate_query_expectation_accepts_when_count_and_names_met() {
        let query = QueryExpectation {
            description: "ok".into(),
            key: key_u32("batch_id", 1),
            min_events: 2,
            expected_event_names: vec!["BurstEmitted".into()],
        };
        let response = json!({"result": {"events": [
            {"event": {"eventName": "BurstEmitted"}},
            {"event": {"eventName": "Other"}},
        ]}});
        assert!(validate_query_expectation(&query, &response).is_ok());
    }

    #[test]
    fn validate_query_expectation_rejects_when_count_below_min() {
        let query = QueryExpectation {
            description: "too few".into(),
            key: key_u32("batch_id", 1),
            min_events: 5,
            expected_event_names: vec![],
        };
        let response = json!({"result": {"events": [{}]}});
        let err = validate_query_expectation(&query, &response).unwrap_err();
        assert!(err.contains("too few"));
        assert!(err.contains("returned 1"));
    }

    #[test]
    fn validate_query_expectation_rejects_missing_expected_event() {
        let query = QueryExpectation {
            description: "missing name".into(),
            key: key_u32("batch_id", 1),
            min_events: 1,
            expected_event_names: vec!["RecordStored".into()],
        };
        let response = json!({"result": {"events": [{"event": {"eventName": "BurstEmitted"}}]}});
        let err = validate_query_expectation(&query, &response).unwrap_err();
        assert!(err.contains("missing decoded event 'RecordStored'"));
    }

    // ─── Manifest / report serde round-trips ───────────────────────────────

    #[test]
    fn query_expectation_serde_round_trip_camel_case() {
        let query = QueryExpectation {
            description: "desc".into(),
            key: key_u32("batch_id", 9),
            min_events: 3,
            expected_event_names: vec!["A".into()],
        };
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("minEvents"));
        let back: QueryExpectation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.description, "desc");
        assert_eq!(back.min_events, 3);
        assert_eq!(back.expected_event_names, vec!["A".to_string()]);
    }

    #[test]
    fn seed_manifest_serde_round_trip() {
        let manifest = SeedManifest {
            mode: "bulk".into(),
            genesis_hash: "abcdef".into(),
            start_block: 1,
            end_block: 100,
            total_blocks: 100,
            transactions_submitted: 50,
            synthetic_event_count: 100,
            queries: vec![],
        };
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("genesisHash"));
        let back: SeedManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.end_block, 100);
        assert_eq!(back.synthetic_event_count, 100);
    }

    #[test]
    fn benchmark_report_serde_round_trip() {
        let report = BenchmarkReport {
            chain_tip: 10,
            indexed_blocks: 9,
            queue_depth: 4,
            synthetic_event_count: 18,
            elapsed_seconds: 1.5,
            blocks_per_second: 6.0,
            synthetic_events_per_second: 12.0,
        };
        let json = serde_json::to_string(&report).unwrap();
        let back: BenchmarkReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.queue_depth, 4);
        assert_eq!(back.blocks_per_second, 6.0);
    }

    #[test]
    fn pick_unused_port_returns_bindable_port() {
        let port = pick_unused_port().unwrap();
        assert!(port > 0);
        // The chosen port must be bindable again to avoid a transient collision.
        let listener = std::net::TcpListener::bind(("127.0.0.1", port)).unwrap();
        drop(listener);
    }
}
