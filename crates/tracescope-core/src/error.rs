//! Core error type.

/// Errors produced by the trace model and analysis.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("trace serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid trace: {0}")]
    Validation(String),
}
