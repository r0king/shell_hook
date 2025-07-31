use crate::app::AppContext;
use crate::cli::WebhookFormat;
use crate::message::StreamMessage;
use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

pub async fn run_webhook_sender(
    context: Arc<AppContext>,
    mut rx: Receiver<StreamMessage>,
) -> Result<()> {
    let mut buffer: Vec<String> = Vec::new();
    let buffer_timeout = Duration::from_secs_f64(context.cli.buffer_timeout);

    loop {
        match tokio::time::timeout(buffer_timeout, rx.recv()).await {
            Ok(Some(StreamMessage::Line(line))) => {
                buffer.push(line);
                if buffer.len() >= context.cli.buffer_size {
                    send_buffered_lines(&context, &mut buffer).await?;
                }
            }
            Ok(Some(StreamMessage::Flush)) => {
                send_buffered_lines(&context, &mut buffer).await?;
            }
            Ok(Some(StreamMessage::CommandFinished)) => {
                send_buffered_lines(&context, &mut buffer).await?;
                break;
            }
            Ok(None) => {
                // Channel closed, send any remaining lines
                send_buffered_lines(&context, &mut buffer).await?;
                break;
            }
            Err(_) => {
                // Timeout elapsed, send buffered lines
                if !buffer.is_empty() {
                    send_buffered_lines(&context, &mut buffer).await?;
                }
            }
        }
    }

    Ok(())
}

pub async fn send_buffered_lines(
    context: &Arc<AppContext>,
    buffer: &mut Vec<String>,
) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }
    let message = buffer.join("\n");
    let result = send_message(context, &message).await;
    buffer.clear();
    result
}

pub async fn send_message(context: &Arc<AppContext>, message: &str) -> Result<()> {
    let payload = create_payload(message, &context.cli.format);
    send_payload(
        &context.client,
        context.cli.webhook_url.as_deref(),
        &payload,
        context.cli.dry_run,
    )
    .await
}

pub fn create_payload(message: &str, format: &WebhookFormat) -> Value {
    match format {
        WebhookFormat::Slack | WebhookFormat::GoogleChat => json!({ "text": message }),
    }
}

pub async fn send_payload(
    client: &Client,
    webhook_url: Option<&str>,
    payload: &Value,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!("[shell_hook] Dry run: Would send payload: {}", payload);
        return Ok(());
    }

    if let Some(url) = webhook_url {
        client
            .post(url)
            .json(payload)
            .send()
            .await?
            .error_for_status()?;
    }
    Ok(())
}
