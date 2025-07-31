use shell_hook::error::AppError;
use shell_hook::message::StreamMessage;
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
    assert_eq!(
        app_error.to_string(),
        "Failed to send message to the channel"
    );
}

#[tokio::test]
async fn test_task_join_error() {
    let task = tokio::spawn(async {
        panic!("Task panicked for testing purposes");
    });

    // Await the task and expect a JoinError
    let join_error = task.await.unwrap_err();
    let app_error = AppError::from(join_error);

    // Assert that the error is the correct variant and that it was caused by a panic
    match app_error {
        AppError::TaskJoin(e) => {
            assert!(e.is_panic());
        }
        _ => panic!("Expected AppError::TaskJoin, but got {:?}", app_error),
    }
}

#[tokio::test]
async fn test_webhook_error() {
    // Create a mock reqwest error
    let reqwest_error = reqwest::Client::new()
        .get("http://invalid-url-that-will-not-resolve")
        .send()
        .await
        .unwrap_err();
    let app_error = AppError::from(reqwest_error);
    assert!(app_error.to_string().starts_with("Webhook request failed"));
}
