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
    #[arg(short, long, value_name = "TITLE")]
    pub title: Option<String>,

    /// Don't execute the command or send webhooks; just print what would be done.
    #[arg(long)]
    pub dry_run: bool,

    /// The format of the webhook payload.
    #[arg(long, value_enum, default_value_t=WebhookFormat::GoogleChat)]
    pub format: WebhookFormat,

    /// Max number of lines to buffer before sending a webhook message.
    #[arg(long, default_value_t = 10, value_name = "COUNT")]
    pub buffer_size: usize,

    /// Max time in seconds to wait before flushing the buffer.
    #[arg(long, default_value_t = 2.0, value_name = "SECONDS")]
    pub buffer_timeout: f64,

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

impl Default for Args {
    fn default() -> Self {
        Self {
            webhook_url: None,
            on_success: None,
            on_failure: None,
            quiet: false,
            title: None,
            dry_run: false,
            format: WebhookFormat::default(),
            buffer_size: 10,
            buffer_timeout: 2.0,
            command: Vec::new(),
        }
    }
}
