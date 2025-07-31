use crate::app::AppContext;
use crate::cli::WebhookFormat;
use crate::error::AppError;
use crate::message::StreamMessage;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Creates a JSON payload for a given message and format.
pub fn create_payload(message: &str, format: &WebhookFormat) -> Value {
    match format {
        WebhookFormat::GoogleChat => json!({ "text": message }),
        WebhookFormat::Slack => json!({ "text": message }),
    }
}

/// Sends a pre-formatted payload to a webhook URL.
pub async fn send_payload(
    client: &Client,
    webhook_url: Option<&str>,
    payload: &Value,
    is_dry_run: bool,
) -> Result<(), AppError> {
    if is_dry_run {
        println!("[DRY RUN] Would send to webhook: {}", payload);
        return Ok(());
    }
    if let Some(url) = webhook_url {
        client.post(url).json(payload).send().await?;
    }
    Ok(())
}

/// A convenience helper to create and send a simple text message.
pub async fn send_message(context: &Arc<AppContext>, message: &str) -> Result<(), AppError> {
    let payload = create_payload(message, &context.args.format);
    send_payload(
        &context.client,
        context.args.webhook_url.as_deref(),
        &payload,
        context.args.dry_run,
    )
    .await
}

/// The core task that receives lines from a channel and sends them to the webhook in batches.
pub async fn run_webhook_sender(context: Arc<AppContext>, mut rx: mpsc::Receiver<StreamMessage>) {
    if context.args.webhook_url.is_none() && !context.args.dry_run {
        // Still need to drain the receiver if no webhook is set, to prevent the sender from blocking.
        while (rx.recv().await).is_some() {}
        return;
    }

    let mut buffer = Vec::new();
    let buffer_timeout = Duration::from_secs_f64(context.args.buffer_timeout);
    let buffer_max_size = context.args.buffer_size;

    loop {
        match tokio::time::timeout(buffer_timeout, rx.recv()).await {
            // Received a line, add to buffer and send if full
            Ok(Some(StreamMessage::Line(line))) => {
                buffer.push(line);
                if buffer.len() >= buffer_max_size {
                    if let Err(e) = send_buffered_lines(&context, &mut buffer).await {
                        eprintln!("[shell_hook] Error sending buffered lines: {}", e);
                    }
                }
            }
            // Timeout elapsed, send what we have
            Err(_) => {
                if let Err(e) = send_buffered_lines(&context, &mut buffer).await {
                    eprintln!(
                        "[shell_hook] Error sending buffered lines on timeout: {}",
                        e
                    );
                }
            }
            // Command finished or channel closed, send remainder and exit
            Ok(Some(StreamMessage::CommandFinished)) | Ok(None) => {
                if let Err(e) = send_buffered_lines(&context, &mut buffer).await {
                    eprintln!("[shell_hook] Error sending final buffered lines: {}", e);
                }
                break;
            }
        }
    }
}

/// Sends the current buffer of lines as a single webhook message.
pub async fn send_buffered_lines(
    context: &Arc<AppContext>,
    buffer: &mut Vec<String>,
) -> Result<(), AppError> {
    if buffer.is_empty() {
        return Ok(());
    }
    let combined_message = buffer.join("\n");
    let full_message = if let Some(title) = &context.args.title {
        format!("[{}] {}", title, combined_message)
    } else {
        combined_message
    };
    send_message(context, &full_message).await?;
    buffer.clear();
    Ok(())
}
