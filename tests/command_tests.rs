use clap::Parser;
use shell_hook::app::AppContext;
use shell_hook::cli::Args;
use shell_hook::command::run_command_and_stream;
use shell_hook::message::StreamMessage;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::mpsc;

fn create_test_context(quiet: bool, command: Vec<&str>) -> Arc<AppContext> {
    let mut raw_args = vec!["shell_hook"];
    if quiet {
        raw_args.push("--quiet");
    }
    raw_args.push("--");
    raw_args.extend(command);

    let args = Args::parse_from(raw_args);

    Arc::new(AppContext {
        args: Arc::new(args),
        client: Client::new(),
    })
}

#[tokio::test]
async fn test_run_command_success() {
    let context = create_test_context(false, vec!["echo", "hello world"]);
    let (tx, mut rx) = mpsc::channel(10);

    let status_result = run_command_and_stream(context, tx).await;
    assert!(status_result.is_ok());
    let status = status_result.unwrap();
    assert!(status.success());

    // Check that the output was sent to the channel
    let mut lines_received = 0;
    while let Some(msg) = rx.recv().await {
        if let StreamMessage::Line(line) = msg {
            assert_eq!(line, "hello world");
            lines_received += 1;
        } else if let StreamMessage::CommandFinished = msg {
            break;
        }
    }
    assert_eq!(lines_received, 1);
}

#[tokio::test]
async fn test_run_command_failure() {
    let context = create_test_context(false, vec!["sh", "-c", "exit 1"]);
    let (tx, mut rx) = mpsc::channel(10);

    let status_result = run_command_and_stream(context, tx).await;
    assert!(status_result.is_ok());
    let status = status_result.unwrap();
    assert_eq!(status.code(), Some(1));

    // Ensure the CommandFinished message is still sent
    let mut finished = false;
    while let Some(msg) = rx.recv().await {
        if let StreamMessage::CommandFinished = msg {
            finished = true;
            break;
        }
    }
    assert!(finished);
}

#[tokio::test]
async fn test_run_command_quiet_mode() {
    let context = create_test_context(true, vec!["echo", "hello world"]);
    let (tx, mut rx) = mpsc::channel(10);

    let status_result = run_command_and_stream(context, tx).await;
    assert!(status_result.is_ok());

    // In quiet mode, no lines should be sent
    let mut lines_received = 0;
    while let Some(msg) = rx.recv().await {
        if let StreamMessage::Line(_) = msg {
            lines_received += 1;
        } else if let StreamMessage::CommandFinished = msg {
            break;
        }
    }
    assert_eq!(lines_received, 0);
}
