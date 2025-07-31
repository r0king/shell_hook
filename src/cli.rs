use clap::{Parser, ValueEnum};

/// A powerful CLI tool to stream command output to webhooks with buffering,
/// custom messages, and multi-platform support.
#[derive(Parser, Debug)]
#[command(
    author, // Reads from Cargo.toml
    version, // Reads from Cargo.toml
    about, // Reads from Cargo.toml's description
    long_about = None
)]
pub struct Args {
    /// The webhook URL to send messages to. Can also be set via the WEBHOOK_URL environment variable.
    #[arg(long, env = "WEBHOOK_URL", value_name = "URL")]
    pub webhook_url: Option<String>,

    /// Custom message to send on command success.
    #[arg(long, value_name = "MESSAGE")]
    pub on_success: Option<String>,

    /// Custom message to send on command failure.
    #[arg(long, value_name = "MESSAGE")]
    pub on_failure: Option<String>,

    /// Suppress streaming of stdout/stderr to the webhook (start/finish messages are still sent).
    #[arg(short, long)]
    pub quiet: bool,

    /// A title to prepend to all messages, e.g., "[My Project]".
    #[arg(short, long, default_value = "", value_name = "TITLE")]
    pub title: String,

    /// Don't execute the command or send webhooks; just print what would be done.
    #[arg(long)]
    pub dry_run: bool,

    /// The format of the webhook payload.
    #[arg(long, value_enum, default_value_t=WebhookFormat::GoogleChat)]
    pub format: WebhookFormat,

    /// The command to execute and stream its output.
    #[arg(required = true, last = true, value_name = "COMMAND")]
    pub command: Vec<String>,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum WebhookFormat {
    #[default]
    GoogleChat,
    Slack,
}
