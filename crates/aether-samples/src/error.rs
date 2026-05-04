use thiserror::Error;

#[derive(Debug, Error)]
pub enum SampleError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Checksum mismatch for {pack_id}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        pack_id: String,
        expected: String,
        actual: String,
    },

    #[error("Pack not found: {0}")]
    PackNotFound(String),

    #[error("Pack already installed: {0}")]
    AlreadyInstalled(String),

    #[error("Storage directory unavailable: {0}")]
    StorageUnavailable(String),

    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),
}
