use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArkError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("rate limited (429), retry after backoff")]
    RateLimited,

    #[error("task failed: {0}")]
    #[allow(dead_code)]
    TaskFailed(String),

    #[error("task expired")]
    #[allow(dead_code)]
    TaskExpired,

    #[error("{0}")]
    #[allow(dead_code)]
    Other(String),
}
