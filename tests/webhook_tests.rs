use httpmock::prelude::*;
use httpmock::MockServer;
use reqwest::Client;
use serde_json::json;
use shell_hook::app::AppContext;
use shell_hook::cli::{Args, WebhookFormat};
use shell_hook::message::StreamMessage;
use shell_hook::webhook::{create_payload, run_webhook_sender, send_buffered_lines, send_payload};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Creates a default AppContext for testing.
fn mock_context(server: &MockServer, dry_run: bool) -> Arc<AppContext> {
    let args = Args {
        webhook_url: Some(server.url("/")),
        dry_run,
        command: vec!["echo".to_string(), "test".to_string()],
        on_success: None,
        on_failure: None,
        quiet: false,
        title: None,
        format: WebhookFormat::GoogleChat,
        buffer_size: 10,
        buffer_timeout: 2.0,
    };

    Arc::new(AppContext {
        args: Arc::new(args),
        client: Client::new(),
    })
}

#[test]
fn test_create_payload_slack() {
    let message = "hello";
    let payload = create_payload(message, &WebhookFormat::Slack);
    assert_eq!(payload, json!({ "text": "hello" }));
}

#[test]
fn test_create_payload_google_chat() {
    let message = "world";
    let payload = create_payload(message, &WebhookFormat::GoogleChat);
    assert_eq!(payload, json!({ "text": "world" }));
}

#[tokio::test]
async fn test_send_payload_dry_run() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/");
        then.status(200);
    });

    let client = Client::new();
    let payload = json!({"text": "test"});

    // This should not send a request
    let _ = send_payload(&client, Some(&server.url("/")), &payload, true).await;

    // Assert that the mock was not called
    mock.assert_hits(0);
}

#[tokio::test]
async fn test_send_buffered_lines() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/");
        then.status(200).body(r#"{"status":"ok"}"#);
    });

    let context = mock_context(&server, false);
    let mut buffer = vec!["line1".to_string(), "line2".to_string()];

    let _ = send_buffered_lines(&context, &mut buffer).await;

    mock.assert();
    assert!(buffer.is_empty());
}

#[tokio::test]
async fn test_run_webhook_sender_sends_on_timeout() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/");
        then.status(200);
    });

    let context = mock_context(&server, false);
    let (tx, rx) = mpsc::channel(100);

    tx.send(StreamMessage::Line("test".to_string()))
        .await
        .unwrap();

    // Run the sender, but timeout before it can complete
    let _ = tokio::time::timeout(
        Duration::from_secs_f64(context.args.buffer_timeout + 1.0),
        run_webhook_sender(context, rx),
    )
    .await;

    // The mock should have been hit once due to the timeout
    mock.assert_hits(1);
}

#[tokio::test]
async fn test_run_webhook_sender_sends_on_buffer_full() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/");
        then.status(200);
    });

    let context = mock_context(&server, false);
    let (tx, rx) = mpsc::channel(100);
    for i in 0..context.args.buffer_size {
        tx.send(StreamMessage::Line(format!("line {}", i)))
            .await
            .unwrap();
    }

    // Run the sender, it should send immediately when the buffer is full
    let _ = tokio::time::timeout(Duration::from_millis(500), run_webhook_sender(context, rx)).await;

    // The mock should have been hit once
    mock.assert_hits(1);
}
