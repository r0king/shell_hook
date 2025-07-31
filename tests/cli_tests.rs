use clap::Parser;
use hook_stream::cli::{Args, WebhookFormat};

#[test]
fn test_cli_args_parsing() {
    let args = Args::parse_from(vec![
        "hook-stream",
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
        "--",
        "ls",
        "-la",
    ]);

    assert_eq!(args.webhook_url, Some("http://localhost".to_string()));
    assert_eq!(args.on_success, Some("Success!".to_string()));
    assert_eq!(args.on_failure, Some("Failure!".to_string()));
    assert!(args.quiet);
    assert_eq!(args.title, "My Test");
    assert!(args.dry_run);
    assert!(matches!(args.format, WebhookFormat::Slack));
    assert_eq!(args.command, vec!["ls", "-la"]);
}

#[test]
fn test_cli_args_defaults() {
    let args = Args::parse_from(vec!["hook-stream", "--", "echo", "hello"]);

    assert_eq!(args.webhook_url, None);
    assert_eq!(args.on_success, None);
    assert_eq!(args.on_failure, None);
    assert!(!args.quiet);
    assert_eq!(args.title, "");
    assert!(!args.dry_run);
    assert!(matches!(args.format, WebhookFormat::GoogleChat));
    assert_eq!(args.command, vec!["echo", "hello"]);
}
