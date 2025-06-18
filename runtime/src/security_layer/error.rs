/// Security errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum SecurityError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid session")]
    InvalidSession,
    #[error("Session expired")]
    SessionExpired,
    #[error("Rate limited")]
    RateLimited,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Encryption error: {0}")]
    EncryptionError(String),
} 