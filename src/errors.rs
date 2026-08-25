use tokio_tungstenite::tungstenite;

#[derive(thiserror::Error, Debug)]
pub enum IndexError {
    #[error("database error")]
    Sled(#[from] sled::Error),
    #[error("connection error")]
    Subxt(Box<subxt::Error>),
    #[error("websocket error")]
    Tungstenite(#[from] tungstenite::Error),
    #[error("parse error")]
    Hex(#[from] hex::FromHexError),
    #[error("block not found: {0}")]
    BlockNotFound(u32),
    #[error(
        "node is pruning historical state at #{block_number}; --state-pruning must be set to archive-canonical"
    )]
    StatePruningMisconfigured { block_number: u32 },
    #[error("RPC error: {0}")]
    RpcError(#[from] subxt::rpcs::Error),
    #[error("codec error")]
    CodecError(#[from] subxt::ext::codec::Error),
    #[error("metadata error")]
    MetadataError(#[from] subxt::error::MetadataTryFromError),
    #[error("block stream error")]
    BlocksError(Box<subxt::error::BlocksError>),
    #[error("block stream closed")]
    BlockStreamClosed,
    #[error("events error")]
    EventsError(Box<subxt::error::EventsError>),
    #[error("at-block error")]
    OnlineClientAtBlockError(Box<subxt::error::OnlineClientAtBlockError>),
    #[error("online client error")]
    OnlineClientError(#[from] subxt::error::OnlineClientError),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("TOML serialization error")]
    TomlSer(#[from] toml::ser::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<subxt::Error> for IndexError {
    fn from(value: subxt::Error) -> Self {
        Self::Subxt(Box::new(value))
    }
}

impl From<subxt::error::BlocksError> for IndexError {
    fn from(value: subxt::error::BlocksError) -> Self {
        Self::BlocksError(Box::new(value))
    }
}

impl From<subxt::error::EventsError> for IndexError {
    fn from(value: subxt::error::EventsError) -> Self {
        Self::EventsError(Box::new(value))
    }
}

impl From<subxt::error::OnlineClientAtBlockError> for IndexError {
    fn from(value: subxt::error::OnlineClientAtBlockError) -> Self {
        Self::OnlineClientAtBlockError(Box::new(value))
    }
}

pub fn internal_error(message: impl Into<String>) -> IndexError {
    IndexError::Internal(message.into())
}

impl IndexError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            IndexError::Subxt(_)
            | IndexError::RpcError(_)
            | IndexError::BlocksError(_)
            | IndexError::BlockStreamClosed
            | IndexError::EventsError(_)
            | IndexError::OnlineClientAtBlockError(_)
            | IndexError::OnlineClientError(_)
            | IndexError::BlockNotFound(_) => true,
            IndexError::Sled(_)
            | IndexError::Tungstenite(_)
            | IndexError::Hex(_)
            | IndexError::StatePruningMisconfigured { .. }
            | IndexError::CodecError(_)
            | IndexError::MetadataError(_)
            | IndexError::Json(_)
            | IndexError::Io(_)
            | IndexError::TomlSer(_)
            | IndexError::Internal(_) => false,
        }
    }
}

pub fn metadata_version(metadata_bytes: &[u8]) -> Option<u8> {
    if metadata_bytes.len() < 5 {
        return None;
    }

    if &metadata_bytes[..4] != b"meta" {
        return None;
    }

    Some(metadata_bytes[4])
}

pub fn unsupported_metadata_error(version: u8, spec_name: &str, spec_version: u64) -> IndexError {
    internal_error(format!(
        "unsupported metadata version v{version} from runtime {spec_name} specVersion {spec_version}; the node may still be syncing early chain history before a runtime upgrade"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_version_reads_prefixed_version_byte() {
        assert_eq!(metadata_version(b"meta\x0e"), Some(14));
        assert_eq!(metadata_version(b"meta\x08"), Some(8));
    }

    #[test]
    fn metadata_version_returns_none_for_short_or_unprefixed() {
        assert_eq!(metadata_version(b""), None);
        assert_eq!(metadata_version(b"met"), None);
        assert_eq!(metadata_version(b"meta"), None);
        assert_eq!(metadata_version(b"META\x0e"), None);
    }

    #[test]
    fn internal() {
        let err = IndexError::Internal("boom".into());
        assert_eq!(err.to_string(), "internal error: boom");
    }

    #[test]
    fn block_not_found_display() {
        let err = IndexError::BlockNotFound(42);
        assert_eq!(err.to_string(), "block not found: 42");
    }

    #[test]
    fn state_pruning_misconfigured_display_mentions_flag() {
        let err = IndexError::StatePruningMisconfigured { block_number: 7 };
        let text = err.to_string();
        assert!(text.contains("#7"));
        assert!(text.contains("archive-canonical"));
    }

    #[test]
    fn is_recoverable_true_for_transient_node_failures() {
        // block-not-found is a transient condition the supervisor retries on.
        assert!(IndexError::BlockNotFound(3).is_recoverable());
    }

    #[test]
    fn is_recoverable_false_for_local_and_config_errors() {
        assert!(!IndexError::Internal("boom".into()).is_recoverable());
        assert!(!IndexError::StatePruningMisconfigured { block_number: 1 }.is_recoverable());

        let json_err = index_error_from_invalid_json();
        assert!(!json_err.is_recoverable());

        let hex_err = IndexError::from(hex::decode("zz").unwrap_err());
        assert!(!hex_err.is_recoverable());

        let io_err = IndexError::Io(std::io::Error::other("io"));
        assert!(!io_err.is_recoverable());
    }

    #[test]
    fn unsupported_metadata_error_includes_runtime_details() {
        let err = unsupported_metadata_error(12, "my-runtime", 555_000);
        let text = err.to_string();
        assert!(text.contains("v12"));
        assert!(text.contains("my-runtime"));
        assert!(text.contains("555000"));
    }

    // Helps construct a `serde_json::Error` without depending on a private
    // constructor: parse deliberately-invalid JSON.
    fn index_error_from_invalid_json() -> IndexError {
        let err: serde_json::Error = serde_json::from_str::<i32>("{").unwrap_err();
        IndexError::from(err)
    }
}
