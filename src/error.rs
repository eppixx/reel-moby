use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Expected a tag")]
    NoTagFound,

    #[error("Unexpected input")]
    MisformedInput,

    /// a serde error
    #[error("Converting error: {0}")]
    Converting(String),

    /// invalid repos show a valid json with 0 tags
    #[error("Given Repo does not exists or has 0 tags.")]
    NoTagsFound,

    /// converting serde error
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    /// error while handling requests
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}
