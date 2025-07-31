use clap::{Parser, ValueEnum};
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(ValueEnum, Clone, Debug, Default)]
enum WebhookFormat {
    #[default]
    GoogleChat,
    Slack,
}
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    on_success: Option<String>,
    #[arg(long)]
    on_failure: Option<String>,
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
    #[arg(short, long, default_value = "")]
    title: String,
    #[arg(long, default_value_t = false)]
    dry_run: bool,
    #[arg(long, value_enum, default_value_t=WebhookFormat::GoogleChat)]
    format: WebhookFormat,
    #[arg(required = true, last = true)]
    command: Vec<String>,
}

enum Message {
    Line(String),
    CommandFinished,
}

/// Creates a JSON payload for a given message and format.
fn create_payload(message: &str, format: &WebhookFormat) -> Value {
    match format {
        WebhookFormat::GoogleChat => json!({ "text": message }),
        WebhookFormat::Slack => json!({ "text": message }),
    }
}

/// Generic function to send a pre-formatted payload.
async fn send_payload(client: &Client, webhook_url: &str, payload: &Value, is_dry_run: bool) {
    if is_dry_run {
        println!("[DRY RUN] Would send to webhook: {}", payload);
        return;
    }
    if let Err(e) = client.post(webhook_url).json(payload).send().await {
        eprintln!("[hook-stream] Error sending to webhook: {}", e);
    }
}

/// Sends a batch of stdout/stderr lines to the configured webhook.
async fn send_stream_batch(client: &Client, webhook_url: &str, batch: &[String], args: &Args) {
    if batch.is_empty() {
        return;
    }

    let combined_message = batch.join("\n");
    let title_prefix = if !args.title.is_empty() {
        format!("[{}] ", args.title)
    } else {
        "".to_string()
    };

    // This function is now ONLY for stream output, not final messages.
    let full_message = format!("{}{}", title_prefix, combined_message);

    let payload = create_payload(&full_message, &args.format);
    send_payload(client, webhook_url, &payload, args.dry_run).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let webhook_url = if args.dry_run {
        "dry-run".to_string()
    } else {
        match env::var("WEBHOOK_URL") {
            Ok(url) => url,
            Err(_) => {
                eprintln!(
                    "[hook-stream] Critical Error: WEBHOOK_URL environment variable must be set."
                );
                std::process::exit(2);
            }
        }
    };
    if args.command.is_empty() {
        eprintln!("[hook-stream] Error: No command provided after '--'.");
        std::process::exit(1);
    }

    let command_str = args.command.join(" ");
    let http_client = Client::new();
    let (tx, mut rx) = mpsc::channel::<Message>(100);

    let sender_args = Arc::new(args);
    let sender_client = http_client.clone();
    let sender_webhook_url = webhook_url.clone();

    let sender_args_clone = Arc::clone(&sender_args);
    // The sender task now uses the refactored function
    let sender_task = tokio::spawn(async move {
        let mut buffer = Vec::new();
        let buffer_timeout = Duration::from_secs(2);

        loop {
            match tokio::time::timeout(buffer_timeout, rx.recv()).await {
                Ok(Some(Message::Line(line))) => {
                    buffer.push(line);
                    if buffer.len() >= 10 {
                        send_stream_batch(
                            &sender_client,
                            &sender_webhook_url,
                            &buffer,
                            &sender_args_clone,
                        )
                        .await;
                        buffer.clear();
                    }
                }
                Err(_) => {
                    if !buffer.is_empty() {
                        send_stream_batch(
                            &sender_client,
                            &sender_webhook_url,
                            &buffer,
                            &sender_args_clone,
                        )
                        .await;
                        buffer.clear();
                    }
                }
                Ok(Some(Message::CommandFinished)) | Ok(None) => {
                    if !buffer.is_empty() {
                        send_stream_batch(
                            &sender_client,
                            &sender_webhook_url,
                            &buffer,
                            &sender_args_clone,
                        )
                        .await;
                        buffer.clear();
                    }
                    break;
                }
            }
        }
    });

    // --- Spawn Command and Start Streaming ---
    let title_prefix = if !sender_args.title.is_empty() {
        format!("[{}] ", sender_args.title)
    } else {
        "".to_string()
    };

    // The start message is now also handled correctly
    let start_message = format!("{}🚀 Starting command: `{}`", title_prefix, command_str);
    println!("{}", start_message);
    let start_payload = create_payload(&start_message, &sender_args.format);
    send_payload(
        &http_client,
        &webhook_url,
        &start_payload,
        sender_args.dry_run,
    )
    .await;

    let mut child = Command::new(&sender_args.command[0])
        .args(&sender_args.command[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // ... (The stdout/stderr reader tasks and child.wait() logic remains IDENTICAL) ...
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    let tx_out = tx.clone();
    let tx_err = tx.clone();
    let quiet_mode = sender_args.quiet;
    let stdout_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            println!("{}", line);
            if !quiet_mode {
                let _ = tx_out.send(Message::Line(line)).await;
            }
        }
    });
    let stderr_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            eprintln!("{}", line);
            if !quiet_mode {
                let _ = tx_err.send(Message::Line(line)).await;
            }
        }
    });
    let status = child.wait().await?;
    let _ = stdout_task.await;
    let _ = stderr_task.await;
    let _ = tx.send(Message::CommandFinished).await;
    let _ = sender_task.await;

    // --- Final Status ---
    // This logic is now cleaner
    let (base_message, _is_error) = match status.code() {
        Some(0) => (
            sender_args
                .on_success
                .clone()
                .unwrap_or_else(|| "✅ Command finished successfully.".to_string()),
            false,
        ),
        Some(code) => (
            sender_args
                .on_failure
                .clone()
                .unwrap_or_else(|| format!("❌ Command failed with exit code {}.", code)),
            true,
        ),
        None => ("❌ Command was terminated by a signal.".to_string(), true),
    };

    let final_message = format!("{}{}", title_prefix, base_message);
    println!("{}", final_message);

    let final_payload = create_payload(&final_message, &sender_args.format);
    send_payload(
        &http_client,
        &webhook_url,
        &final_payload,
        sender_args.dry_run,
    )
    .await;

    if let Some(code) = status.code() {
        std::process::exit(code);
    }

    Ok(())
}
