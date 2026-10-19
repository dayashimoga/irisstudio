use thiserror::Error;

pub type IrisResult<T> = Result<T, IrisError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum IrisError {
    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Unsupported format or container: {0}")]
    UnsupportedFormat(String),

    #[error("Corrupt or invalid media stream: {0}")]
    CorruptMedia(String),

    #[error("Decoding failed: {0}")]
    DecodeError(String),

    #[error("Encoding failed: {0}")]
    EncodeError(String),

    #[error("Timeline validation failed: {0}")]
    TimelineError(String),

    #[error("Memory or cache budget exceeded: {0}")]
    BudgetExceeded(String),

    #[error("Hardware acceleration unavailable: {0}")]
    HardwareAccelerationUnavailable(String),

    #[error("Operation cancelled by user: {0}")]
    Cancelled(String),

    #[error("Project serialization or recovery failed: {0}")]
    ProjectError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}
