use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// no valid tag found
    #[error("Expected a tag")]
    NoTagFound,

    /// input that can't be interpreted
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

    /// error while sending to channel
    #[error("sending to channel error: {0}")]
    ChannelSendError(#[from] std::sync::mpsc::SendError<crate::ui::UiEvent>),
}
