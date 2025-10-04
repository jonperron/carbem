use thiserror::Error;

/// The main error type for the Carbem library.
#[derive(Error, Debug)]
pub enum CarbemError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Provider-specific error
    #[error("Provider error: {0}")]
    Provider(String),

    /// Unsupported provider
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimit,

    /// Generic error
    #[error("An error occurred: {0}")]
    Other(String),
}

/// Type alias for Result with CarbemError
pub type Result<T> = std::result::Result<T, CarbemError>;
