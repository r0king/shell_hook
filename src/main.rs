use clap::{Parser, ValueEnum};
use reqwest::Client;
use serde_json::{json, Value};
use std::process::{ExitStatus, Stdio};
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Missing Webhook URL: Set --webhook-url or the WEBHOOK_URL environment variable.")]
    MissingWebhookUrl,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Mpsc(#[from] tokio::sync::mpsc::error::SendError<StreamMessage>),

    #[error(transparent)]
    TaskJoin(#[from] tokio::task::JoinError),
}

/// A powerful CLI tool to stream command output to webhooks with buffering,
/// custom messages, and multi-platform support.
#[derive(Parser, Debug)]
#[command(
    author, // Reads from Cargo.toml
    version, // Reads from Cargo.toml
    about, // Reads from Cargo.toml's description
    long_about = None
)]
struct Args {
    /// The webhook URL to send messages to. Can also be set via the WEBHOOK_URL environment variable.
    #[arg(long, env = "WEBHOOK_URL", value_name = "URL")]
    webhook_url: Option<String>,

    /// Custom message to send on command success.
    #[arg(long, value_name = "MESSAGE")]
    on_success: Option<String>,

    /// Custom message to send on command failure.
    #[arg(long, value_name = "MESSAGE")]
    on_failure: Option<String>,

    /// Suppress streaming of stdout/stderr to the webhook (start/finish messages are still sent).
    #[arg(short, long)]
    quiet: bool,

    /// A title to prepend to all messages, e.g., "[My Project]".
    #[arg(short, long, default_value = "", value_name = "TITLE")]
    title: String,

    /// Don't execute the command or send webhooks; just print what would be done.
    #[arg(long)]
    dry_run: bool,

    /// The format of the webhook payload.
    #[arg(long, value_enum, default_value_t=WebhookFormat::GoogleChat)]
    format: WebhookFormat,

    /// The command to execute and stream its output.
    #[arg(required = true, last = true, value_name = "COMMAND")]
    command: Vec<String>,
}

#[derive(ValueEnum, Clone, Debug, Default)]
enum WebhookFormat {
    #[default]
    GoogleChat,
    Slack,
}

/// An enum to pass messages from the command runners to the webhook sender.
#[derive(Clone)]
pub enum StreamMessage {
    Line(String),
    CommandFinished,
}

/// Shared application context to avoid passing many arguments.
struct AppContext {
    args: Args,
    client: Client,
    webhook_url: Option<String>,
    title_prefix: String,
}

/// Creates a JSON payload for a given message and format.
fn create_payload(message: &str, format: &WebhookFormat) -> Value {
    match format {
        WebhookFormat::GoogleChat => json!({ "text": message }),
        WebhookFormat::Slack => json!({ "text": message }),
    }
}

/// Sends a pre-formatted payload to a webhook URL.
async fn send_payload(client: &Client, webhook_url: &str, payload: &Value, is_dry_run: bool) {
    if is_dry_run {
        println!("[DRY RUN] Would send to webhook: {}", payload);
        return;
    }
    if let Err(e) = client.post(webhook_url).json(payload).send().await {
        eprintln!("[hook-stream] Error sending to webhook: {}", e);
    }
}

/// A convenience helper to create and send a simple text message.
async fn send_message(context: &Arc<AppContext>, message: &str) {
    if let Some(url) = &context.webhook_url {
        let payload = create_payload(message, &context.args.format);
        send_payload(&context.client, url, &payload, context.args.dry_run).await;
    }
}

/// The core task that receives lines from a channel and sends them to the webhook in batches.
async fn run_webhook_sender(context: Arc<AppContext>, mut rx: mpsc::Receiver<StreamMessage>) {
    if context.webhook_url.is_none() && !context.args.dry_run {
        // Still need to drain the receiver if no webhook is set, to prevent the sender from blocking.
        while let Some(_) = rx.recv().await {}
        return;
    }

    let mut buffer = Vec::new();
    let buffer_timeout = Duration::from_secs(2);
    let buffer_max_size = 10;

    loop {
        match tokio::time::timeout(buffer_timeout, rx.recv()).await {
            // Received a line, add to buffer and send if full
            Ok(Some(StreamMessage::Line(line))) => {
                buffer.push(line);
                if buffer.len() >= buffer_max_size {
                    send_buffered_lines(&context, &mut buffer).await;
                }
            }
            // Timeout elapsed, send what we have
            Err(_) => {
                send_buffered_lines(&context, &mut buffer).await;
            }
            // Command finished or channel closed, send remainder and exit
            Ok(Some(StreamMessage::CommandFinished)) | Ok(None) => {
                send_buffered_lines(&context, &mut buffer).await;
                break;
            }
        }
    }
}

/// Sends the current buffer of lines as a single webhook message.
async fn send_buffered_lines(context: &Arc<AppContext>, buffer: &mut Vec<String>) {
    if buffer.is_empty() {
        return;
    }
    let combined_message = buffer.join("\n");
    let full_message = format!("{}{}", context.title_prefix, combined_message);
    send_message(context, &full_message).await;
    buffer.clear();
}

/// Spawns the command, captures its stdout/stderr, and sends lines to the channel.
async fn run_command_and_stream(
    context: Arc<AppContext>,
    tx: mpsc::Sender<StreamMessage>,
) -> std::io::Result<ExitStatus> {
    let mut child = Command::new(&context.args.command[0])
        .args(&context.args.command[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Spawn tasks to read stdout and stderr concurrently
    let tx_out = tx.clone();
    let quiet_mode = context.args.quiet;
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            println!("{}", line);
            if !quiet_mode {
                if tx_out.send(StreamMessage::Line(line)).await.is_err() {
                    break; // Receiver has been dropped
                }
            }
        }
    });

    let tx_err = tx.clone();
    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            eprintln!("{}", line);
            if !quiet_mode {
                if tx_err.send(StreamMessage::Line(line)).await.is_err() {
                    break; // Receiver has been dropped
                }
            }
        }
    });

    // Wait for the command to complete and for readers to finish
    let status = child.wait().await?;
    let _ = stdout_task.await;
    let _ = stderr_task.await;

    // Signal that the command is done
    let _ = tx.send(StreamMessage::CommandFinished).await;

    Ok(status)
}

async fn run() -> Result<i32, AppError> {
    let args = Args::parse();

    // Validate arguments
    if args.webhook_url.is_none() && !args.dry_run {
        return Err(AppError::MissingWebhookUrl);
    }

    let command_str = args.command.join(" ");
    let title_prefix = if !args.title.is_empty() {
        format!("[{}] ", args.title)
    } else {
        "".to_string()
    };

    // Create shared context
    let context = Arc::new(AppContext {
        webhook_url: args.webhook_url.clone(),
        args,
        client: Client::new(),
        title_prefix,
    });

    // --- Setup communication channel and tasks ---
    let (tx, rx) = mpsc::channel::<StreamMessage>(100);
    let sender_task = tokio::spawn(run_webhook_sender(context.clone(), rx));

    // --- Send initial message ---
    let start_message = format!(
        "{}🚀 Starting command: `{}`",
        context.title_prefix, command_str
    );
    println!("{}", start_message);
    send_message(&context, &start_message).await;

    // --- Run command and stream output ---
    let status_result = run_command_and_stream(context.clone(), tx).await;

    // --- Wait for sender to finish sending buffered messages ---
    sender_task.await?;

    // --- Send final status message ---
    match status_result {
        Ok(status) => {
            let exit_code = status.code().unwrap_or(1);
            let (base_message, is_error) = match status.code() {
                Some(0) => (
                    context
                        .args
                        .on_success
                        .clone()
                        .unwrap_or_else(|| "✅ Command finished successfully.".to_string()),
                    false,
                ),
                Some(code) => (
                    context
                        .args
                        .on_failure
                        .clone()
                        .unwrap_or_else(|| format!("❌ Command failed with exit code {}.", code)),
                    true,
                ),
                None => ("❌ Command was terminated by a signal.".to_string(), true),
            };

            let final_message = format!("{}{}", context.title_prefix, base_message);
            if is_error {
                eprintln!("{}", final_message);
            } else {
                println!("{}", final_message);
            }
            send_message(&context, &final_message).await;
            Ok(exit_code)
        }
        Err(e) => {
            let base_message = context.args.on_failure.clone().unwrap_or_else(|| {
                format!("❌ Command failed to start: {}.", e)
            });
            let final_message = format!("{}{}", context.title_prefix, base_message);
            eprintln!("{}", final_message);
            send_message(&context, &final_message).await;
            // Decide on an exit code for command start failure
            match e.kind() {
                ErrorKind::NotFound => Ok(127),
                _ => Ok(1),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let result = run().await;

    if let Err(e) = &result {
        eprintln!("[hook-stream] Error: {}", e);
    }

    std::process::exit(match result {
        Ok(code) => code,
        Err(_) => 1,
    });
}
