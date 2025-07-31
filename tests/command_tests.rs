use reqwest::Client;
use shell_hook::app::AppContext;
use shell_hook::cli::{Args, WebhookFormat};
use shell_hook::command::run_command_and_stream;
use shell_hook::message::StreamMessage;
use std::sync::Arc;
use tokio::sync::mpsc;

fn create_test_context(command: Vec<&str>, quiet: bool) -> Arc<AppContext> {
    let args = Args {
        command: command.into_iter().map(String::from).collect(),
        quiet,
        webhook_url: None,
        on_success: None,
        on_failure: None,
        title: None,
        dry_run: false,
        format: WebhookFormat::GoogleChat,
        buffer_size: 10,
        buffer_timeout: 2.0,
    };
    Arc::new(AppContext {
        args: Arc::new(args),
        client: Client::new(),
    })
}

async fn collect_messages(mut rx: mpsc::Receiver<StreamMessage>) -> Vec<StreamMessage> {
    let mut messages = Vec::new();
    while let Some(msg) = rx.recv().await {
        if let StreamMessage::CommandFinished = msg {
            break;
        }
        messages.push(msg);
    }
    messages
}

#[tokio::test]
async fn test_run_command_success() {
    let context = create_test_context(vec!["echo", "hello world"], false);
    let (tx, rx) = mpsc::channel(10);

    let status_result = run_command_and_stream(context, tx).await;
    assert!(status_result.is_ok());
    let status = status_result.unwrap();
    assert!(status.success());

    let messages = collect_messages(rx).await;
    assert_eq!(messages.len(), 1);
    if let Some(StreamMessage::Line(line)) = messages.get(0) {
        assert_eq!(line, "hello world");
    } else {
        panic!("Expected a Line message");
    }
}

#[tokio::test]
async fn test_run_command_with_stderr() {
    let context = create_test_context(vec!["sh", "-c", "echo 'error message' >&2"], false);
    let (tx, rx) = mpsc::channel(10);
    let status_result = run_command_and_stream(context, tx).await;
    assert!(status_result.is_ok());
    let status = status_result.unwrap();
    assert!(status.success());
    let messages = collect_messages(rx).await;
    assert_eq!(messages.len(), 1);
    if let Some(StreamMessage::Line(line)) = messages.get(0) {
        assert_eq!(line, "error message");
    } else {
        panic!("Expected a Line message with stderr content");
    }
}

#[tokio::test]
async fn test_run_command_failure() {
    let context = create_test_context(vec!["sh", "-c", "exit 1"], false);
    let (tx, rx) = mpsc::channel(10);

    let status_result = run_command_and_stream(context, tx).await;
    assert!(status_result.is_ok());
    let status = status_result.unwrap();
    assert_eq!(status.code(), Some(1));

    let messages = collect_messages(rx).await;
    assert!(messages.is_empty());
}

#[tokio::test]
async fn test_run_command_quiet_mode() {
    let context = create_test_context(vec!["echo", "hello world"], true);
    let (tx, rx) = mpsc::channel(10);

    let status_result = run_command_and_stream(context, tx).await;
    assert!(status_result.is_ok());

    let messages = collect_messages(rx).await;
    assert!(messages.is_empty());
}
