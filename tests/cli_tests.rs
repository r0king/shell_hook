use clap::Parser;
use shell_hook::cli::{Args, WebhookFormat};
use std::env;

#[test]
fn test_cli_args_parsing() {
    let args = Args::parse_from(vec![
        "shell_hook",
        "--webhook-url",
        "http://localhost",
        "--on-success",
        "Success!",
        "--on-failure",
        "Failure!",
        "-q",
        "-t",
        "My Test",
        "--dry-run",
        "--format",
        "slack",
        "--buffer-size",
        "20",
        "--buffer-timeout",
        "5.0",
        "--",
        "ls",
        "-la",
    ]);

    assert_eq!(args.webhook_url, Some("http://localhost".to_string()));
    assert_eq!(args.on_success, Some("Success!".to_string()));
    assert_eq!(args.on_failure, Some("Failure!".to_string()));
    assert!(args.quiet);
    assert_eq!(args.title, Some("My Test".to_string()));
    assert!(args.dry_run);
    assert!(matches!(args.format, WebhookFormat::Slack));
    assert_eq!(args.command, vec!["ls", "-la"]);
    assert_eq!(args.buffer_size, 20);
    assert_eq!(args.buffer_timeout, 5.0);
}

#[test]
fn test_cli_args_defaults() {
    // Temporarily clear the environment variable to test defaults
    let original_webhook_url = env::var("WEBHOOK_URL").ok();
    env::remove_var("WEBHOOK_URL");

    let args = Args::parse_from(vec!["shell_hook", "--", "echo", "hello"]);

    // Restore the environment variable after parsing
    if let Some(url) = original_webhook_url {
        env::set_var("WEBHOOK_URL", url);
    }

    assert_eq!(args.webhook_url, None);
    assert_eq!(args.on_success, None);
    assert_eq!(args.on_failure, None);
    assert!(!args.quiet);
    assert_eq!(args.title, None);
    assert!(!args.dry_run);
    assert!(matches!(args.format, WebhookFormat::GoogleChat));
    assert_eq!(args.command, vec!["echo", "hello"]);
    assert_eq!(args.buffer_size, 10);
    assert_eq!(args.buffer_timeout, 2.0);
}
