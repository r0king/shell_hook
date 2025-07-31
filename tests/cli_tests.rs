use clap::Parser;
use shell_hook::cli::{Cli, Command, RunArgs, WebhookFormat};
use std::env;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref ENV_LOCK: Mutex<()> = Mutex::new(());
}

#[test]
fn test_cli_args_parsing() {
    let _lock = ENV_LOCK.lock().unwrap();
    let cli = Cli::parse_from(vec![
        "shell_hook",
        "--webhook-url",
        "http://localhost",
        "-t",
        "My Test",
        "--dry-run",
        "--format",
        "slack",
        "--buffer-size",
        "20",
        "--buffer-timeout",
        "5.0",
        "run",
        "--on-success",
        "Success!",
        "--on-failure",
        "Failure!",
        "-q",
        "--",
        "ls",
        "-la",
    ]);

    assert_eq!(cli.webhook_url, Some("http://localhost".to_string()));
    assert_eq!(cli.title, Some("My Test".to_string()));
    assert!(cli.dry_run);
    assert!(matches!(cli.format, WebhookFormat::Slack));
    assert_eq!(cli.buffer_size, 20);
    assert_eq!(cli.buffer_timeout, 5.0);

    if let Command::Run(run_args) = cli.command {
        assert_eq!(run_args.on_success, Some("Success!".to_string()));
        assert_eq!(run_args.on_failure, Some("Failure!".to_string()));
        assert!(run_args.quiet);
        assert_eq!(run_args.command, vec!["ls", "-la"]);
    } else {
        panic!("Expected Command::Run");
    }
}

#[test]
fn test_cli_args_defaults() {
    let _lock = ENV_LOCK.lock().unwrap();
    // Temporarily clear the environment variable to test defaults
    let original_webhook_url = env::var("WEBHOOK_URL").ok();
    env::remove_var("WEBHOOK_URL");

    let cli = Cli::parse_from(vec!["shell_hook", "run", "--", "echo", "hello"]);

    // Restore the environment variable after parsing
    if let Some(url) = original_webhook_url {
        env::set_var("WEBHOOK_URL", url);
    }

    assert_eq!(cli.webhook_url, None);
    assert_eq!(cli.title, None);
    assert!(!cli.dry_run);
    assert!(matches!(cli.format, WebhookFormat::GoogleChat));
    assert_eq!(cli.buffer_size, 10);
    assert_eq!(cli.buffer_timeout, 2.0);

    if let Command::Run(run_args) = cli.command {
        assert_eq!(run_args.on_success, None);
        assert_eq!(run_args.on_failure, None);
        assert!(!run_args.quiet);
        assert_eq!(run_args.command, vec!["echo", "hello"]);
    } else {
        panic!("Expected Command::Run");
    }
}

#[test]
fn test_shell_subcommand() {
    let _lock = ENV_LOCK.lock().unwrap();
    let cli = Cli::parse_from(vec!["shell_hook", "shell"]);
    assert!(matches!(cli.command, Command::Shell));
}

#[test]
fn test_webhook_url_from_env() {
    let _lock = ENV_LOCK.lock().unwrap();
    let webhook_url = "http://localhost/from-env";
    env::set_var("WEBHOOK_URL", webhook_url);

    let cli = Cli::parse_from(vec!["shell_hook", "run", "--", "echo", "hello"]);

    assert_eq!(cli.webhook_url, Some(webhook_url.to_string()));

    env::remove_var("WEBHOOK_URL");
}

#[test]
fn test_webhook_format_enum() {
    let _lock = ENV_LOCK.lock().unwrap();
    let cli_google = Cli::parse_from(vec![
        "shell_hook",
        "--format",
        "google-chat",
        "run",
        "--",
        "echo",
        "hello",
    ]);
    assert!(matches!(cli_google.format, WebhookFormat::GoogleChat));

    let cli_slack = Cli::parse_from(vec![
        "shell_hook",
        "--format",
        "slack",
        "run",
        "--",
        "echo",
        "hello",
    ]);
    assert!(matches!(cli_slack.format, WebhookFormat::Slack));
}

#[test]
fn test_derived_traits() {
    // Test Debug trait
    let run_args = RunArgs {
        on_success: Some("Success".to_string()),
        on_failure: Some("Failure".to_string()),
        quiet: true,
        command: vec!["ls".to_string()],
    };
    let cli = Cli {
        command: Command::Run(run_args.clone()),
        webhook_url: Some("http://localhost".to_string()),
        title: Some("Title".to_string()),
        format: WebhookFormat::Slack,
        buffer_size: 20,
        buffer_timeout: 5.0,
        dry_run: true,
    };
    println!("{:?}", cli);
    println!("{:?}", run_args);
    println!("{:?}", Command::Shell);

    // Test Clone trait for RunArgs
    let run_args_clone = run_args.clone();
    assert_eq!(run_args.on_success, run_args_clone.on_success);

    // Test Default trait for WebhookFormat
    let default_format = WebhookFormat::default();
    assert!(matches!(default_format, WebhookFormat::GoogleChat));
}

#[test]
fn test_cli_long_about() {
    let _lock = ENV_LOCK.lock().unwrap();
    let cli = Cli::parse_from(vec!["shell_hook", "help"]);
    let help_text = format!("{:?}", cli);
    assert!(help_text.contains("A powerful CLI tool to stream command output to webhooks"));
}

#[test]
fn test_run_subcommand_help() {
    let _lock = ENV_LOCK.lock().unwrap();
    let cli = Cli::parse_from(vec!["shell_hook", "help", "run"]);
    let help_text = format!("{:?}", cli);
    assert!(help_text.contains("Run a single command and stream its output"));
    assert!(help_text.contains("--on-success"));
    assert!(help_text.contains("--on-failure"));
    assert!(help_text.contains("--quiet"));
}
