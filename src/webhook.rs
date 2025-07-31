use crate::app::AppContext;
use crate::cli::WebhookFormat;
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
pub async fn send_payload(client: &Client, webhook_url: &str, payload: &Value, is_dry_run: bool) {
    if is_dry_run {
        println!("[DRY RUN] Would send to webhook: {}", payload);
        return;
    }
    if let Err(e) = client.post(webhook_url).json(payload).send().await {
        eprintln!("[hook-stream] Error sending to webhook: {}", e);
    }
}

/// A convenience helper to create and send a simple text message.
pub async fn send_message(context: &Arc<AppContext>, message: &str) {
    if let Some(url) = context.args.webhook_url.as_deref() {
        let payload = create_payload(message, &context.args.format);
        send_payload(&context.client, url, &payload, context.args.dry_run).await;
    }
}

/// The core task that receives lines from a channel and sends them to the webhook in batches.
pub async fn run_webhook_sender(context: Arc<AppContext>, mut rx: mpsc::Receiver<StreamMessage>) {
    if context.args.webhook_url.is_none() && !context.args.dry_run {
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
pub async fn send_buffered_lines(context: &Arc<AppContext>, buffer: &mut Vec<String>) {
    if buffer.is_empty() {
        return;
    }
    let combined_message = buffer.join("\n");
    let title_prefix = if !context.args.title.is_empty() {
        format!("[{}] ", context.args.title)
    } else {
        "".to_string()
    };
    let full_message = format!("{}{}", title_prefix, combined_message);
    send_message(context, &full_message).await;
    buffer.clear();
}
