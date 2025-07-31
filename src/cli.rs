use clap::{Parser, Subcommand, ValueEnum};

/// A powerful CLI tool to stream command output to webhooks with buffering,
/// custom messages, and multi-platform support.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// The webhook URL to send messages to. Can also be set via the WEBHOOK_URL environment variable.
    #[arg(long, global = true, env = "WEBHOOK_URL", value_name = "URL")]
    pub webhook_url: Option<String>,

    /// A title to prepend to all messages, e.g., "[My Project]".
    #[arg(short, long, global = true, value_name = "TITLE")]
    pub title: Option<String>,

    /// The format of the webhook payload.
    #[arg(long, global = true, value_enum, default_value_t=WebhookFormat::GoogleChat)]
    pub format: WebhookFormat,

    /// Max number of lines to buffer before sending a webhook message.
    #[arg(long, global = true, default_value_t = 10, value_name = "COUNT")]
    pub buffer_size: usize,

    /// Max time in seconds to wait before flushing the buffer.
    #[arg(long, global = true, default_value_t = 2.0, value_name = "SECONDS")]
    pub buffer_timeout: f64,

    /// Don't execute the command or send webhooks; just print what would be done.
    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run a single command and stream its output.
    Run(RunArgs),
    /// Start an interactive shell session.
    Shell,
}

/// Arguments for running a single command.
#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
    /// Custom message to send on command success.
    #[arg(long, value_name = "MESSAGE")]
    pub on_success: Option<String>,

    /// Custom message to send on command failure.
    #[arg(long, value_name = "MESSAGE")]
    pub on_failure: Option<String>,

    /// Suppress streaming of stdout/stderr to the webhook (start/finish messages are still sent).
    #[arg(short, long)]
    pub quiet: bool,

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
