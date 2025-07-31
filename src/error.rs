use crate::message::StreamMessage;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Missing Webhook URL: Set --webhook-url or the WEBHOOK_URL environment variable.")]
    MissingWebhookUrl,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Failed to send message to the channel")]
    MpscSendError(#[from] tokio::sync::mpsc::error::SendError<StreamMessage>),

    #[error(transparent)]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("Webhook request failed: {0}")]
    WebhookError(#[from] reqwest::Error),
}
