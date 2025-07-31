use shell_hook::app::{format_with_title, run_app, run_single_command, AppContext};
use shell_hook::cli::{Cli, Command, WebhookFormat};
use shell_hook::error::AppError;

use httpmock::prelude::*;
use std::sync::Arc;

// Helper function to create a Cli instance from args
fn try_cli_from(args: &[&str]) -> Result<Cli, clap::Error> {
    use clap::Parser;
    Cli::try_parse_from(args)
}

#[tokio::test]
async fn test_run_single_command_success() {
    let server = MockServer::start();
    let webhook_url = server.url("/webhook");
    server.mock(|when, then| {
        when.method(POST).path("/webhook");
        then.status(200);
    });

    let cli = try_cli_from(&[
        "shell_hook",
        "--webhook-url",
        &webhook_url,
        "run",
        "--",
        "echo",
        "hello",
    ])
    .unwrap();

    let context = Arc::new(AppContext {
        cli: Arc::new(cli),
        client: reqwest::Client::new(),
    });

    let run_args = match &context.cli.command {
        Command::Run(args) => args,
        _ => panic!("Expected Run command"),
    };

    let result = run_single_command(&context, run_args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[tokio::test]
async fn test_run_single_command_failure() {
    let server = MockServer::start();
    let webhook_url = server.url("/webhook");
    server.mock(|when, then| {
        when.method(POST).path("/webhook");
        then.status(200);
    });

    let cli = try_cli_from(&[
        "shell_hook",
        "--webhook-url",
        &webhook_url,
        "run",
        "--",
        "non_existent_command",
    ])
    .unwrap();

    let context = Arc::new(AppContext {
        cli: Arc::new(cli),
        client: reqwest::Client::new(),
    });

    let run_args = match &context.cli.command {
        Command::Run(args) => args,
        _ => panic!("Expected Run command"),
    };

    let result = run_single_command(&context, run_args).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 127);
}

#[tokio::test]
async fn test_run_missing_webhook_url() {
    let cli = try_cli_from(&["shell_hook", "run", "--", "echo", "hello"]).unwrap();
    let result = run_app(cli).await;
    assert!(result.is_err());
    match result.err().unwrap() {
        AppError::MissingWebhookUrl => {}
        e => panic!("Expected MissingWebhookUrl error, got {:?}", e),
    }
}

#[test]
fn test_format_with_title() {
    let cli_with_title = Cli {
        title: Some("MyTitle".to_string()),
        command: Command::Shell,
        webhook_url: None,
        format: WebhookFormat::GoogleChat,
        buffer_size: 10,
        buffer_timeout: 2.0,
        dry_run: false,
    };
    let cli_without_title = Cli {
        title: None,
        command: Command::Shell,
        webhook_url: None,
        format: WebhookFormat::GoogleChat,
        buffer_size: 10,
        buffer_timeout: 2.0,
        dry_run: false,
    };

    let message = "Test message";

    assert_eq!(
        format_with_title(&cli_with_title, message),
        "[MyTitle] Test message"
    );
    assert_eq!(
        format_with_title(&cli_without_title, message),
        "Test message"
    );
}