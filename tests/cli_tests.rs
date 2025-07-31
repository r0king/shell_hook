use clap::Parser;
use shell_hook::cli::{Cli, Command, WebhookFormat};
use std::env;

#[test]
fn test_cli_args_parsing() {
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
    let cli = Cli::parse_from(vec!["shell_hook", "shell"]);
    assert!(matches!(cli.command, Command::Shell));
}
