use hook_stream::error::AppError;
use hook_stream::message::StreamMessage;
use std::io;
use tokio::sync::mpsc;

#[test]
fn test_missing_webhook_url_error() {
    let error = AppError::MissingWebhookUrl;
    assert_eq!(
        error.to_string(),
        "Missing Webhook URL: Set --webhook-url or the WEBHOOK_URL environment variable."
    );
}

#[test]
fn test_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let app_error = AppError::Io(io_error);
    assert_eq!(app_error.to_string(), "file not found");
}

#[tokio::test]
async fn test_mpsc_error() {
    let (tx, rx) = mpsc::channel::<StreamMessage>(1);
    // Create an error by closing the receiver
    drop(rx);
    let send_error = tx.send(StreamMessage::CommandFinished).await.unwrap_err();
    let app_error = AppError::from(send_error);
    assert_eq!(app_error.to_string(), "channel closed");
}
