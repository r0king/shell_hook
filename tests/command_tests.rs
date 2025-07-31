use reqwest::Client;
use shell_hook::app::AppContext;
use shell_hook::cli::{Cli, Command, RunArgs, WebhookFormat};
use shell_hook::command::run_command_and_stream;
use shell_hook::message::StreamMessage;
use std::sync::Arc;
use tokio::sync::mpsc;

fn create_test_context(run_args: RunArgs) -> (Arc<AppContext>, RunArgs) {
    let cli = Cli {
        command: Command::Run(run_args.clone()),
        webhook_url: None,
        title: None,
        dry_run: false,
        format: WebhookFormat::GoogleChat,
        buffer_size: 10,
        buffer_timeout: 2.0,
    };
    let context = Arc::new(AppContext {
        cli: Arc::new(cli),
        client: Client::new(),
    });
    (context, run_args)
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
    let run_args = RunArgs {
        command: vec!["echo hello world".to_string()],
        quiet: false,
        on_success: None,
        on_failure: None,
    };
    let (context, run_args) = create_test_context(run_args);
    let (tx, rx) = mpsc::channel(10);

    let status_result = run_command_and_stream(context, tx, &run_args).await;
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
    let run_args = RunArgs {
        command: vec!["echo 'error message' >&2".to_string()],
        quiet: false,
        on_success: None,
        on_failure: None,
    };
    let (context, run_args) = create_test_context(run_args);
    let (tx, rx) = mpsc::channel(10);
    let status_result = run_command_and_stream(context, tx, &run_args).await;
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
    let run_args = RunArgs {
        command: vec!["exit 1".to_string()],
        quiet: false,
        on_success: None,
        on_failure: None,
    };
    let (context, run_args) = create_test_context(run_args);
    let (tx, rx) = mpsc::channel(10);

    let status_result = run_command_and_stream(context, tx, &run_args).await;
    assert!(status_result.is_ok());
    let status = status_result.unwrap();
    assert_eq!(status.code(), Some(1));

    let messages = collect_messages(rx).await;
    assert!(messages.is_empty());
}

#[tokio::test]
async fn test_run_command_quiet_mode() {
    let run_args = RunArgs {
        command: vec!["echo hello world".to_string()],
        quiet: true,
        on_success: None,
        on_failure: None,
    };
    let (context, run_args) = create_test_context(run_args);
    let (tx, rx) = mpsc::channel(10);

    let status_result = run_command_and_stream(context, tx, &run_args).await;
    assert!(status_result.is_ok());

    let messages = collect_messages(rx).await;
    assert!(messages.is_empty());
}
