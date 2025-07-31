use clap::Parser;
use reqwest::Client;
use serde_json::json;
use std::env;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// A lightweight CLI tool to stream a command's output to a webhook.
/// The webhook URL is configured via the WEBHOOK_URL environment variable.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The command to execute and stream. Must be passed after '--'.
    #[arg(required = true, last = true)]
    command: Vec<String>,
}

/// Sends a message to the configured webhook.
/// For Google Chat, we need a JSON payload like `{"text": "your message"}`.
async fn send_to_webhook(client: &Client, webhook_url: &str, message: &str, is_error: bool) {
    let formatted_message = if is_error {
        format!("❌ ERROR:\n```\n{}\n```", message)
    } else {
        format!("```\n{}\n```", message)
    };

    let payload = json!({ "text": formatted_message });

    // Send the request and handle potential errors.
    if let Err(e) = client.post(webhook_url).json(&payload).send().await {
        // If we can't send to the webhook, print the error to our own stderr.
        eprintln!("[hook-stream] Error sending to webhook: {}", e);
    }
}

// The main function uses tokio's async runtime.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Configuration and Argument Parsing ---
    let args = Args::parse();

    // Retrieve the webhook URL from the environment.
    let webhook_url = match env::var("WEBHOOK_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("[hook-stream] Critical Error: WEBHOOK_URL environment variable must be set.");
            // Use a specific exit code for configuration errors.
            std::process::exit(2);
        }
    };

    if args.command.is_empty() {
        eprintln!("[hook-stream] Error: No command provided after '--'.");
        std::process::exit(1);
    }

    let command_str = args.command.join(" ");
    let http_client = Client::new();

    // --- Spawn Command and Start Streaming ---
    let start_message = format!("🚀 Starting command: {}", command_str);
    println!("{}", start_message); // Also print locally
    send_to_webhook(&http_client, &webhook_url, &start_message, false).await;

    // Spawn the child command.
    let mut child = Command::new(&args.command[0])
        .args(&args.command[1..])
        .stdout(Stdio::piped()) // Capture stdout
        .stderr(Stdio::piped()) // Capture stderr
        .spawn()?;

    // Get handles to the output streams.
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    // Use BufReader for efficient line-by-line reading.
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    // Asynchronously read from both stdout and stderr at the same time.
    loop {
        tokio::select! {
            // Read a line from stdout
            Ok(Some(line)) = stdout_reader.next_line() => {
                println!("{}", line);
                send_to_webhook(&http_client, &webhook_url, &line, false).await;
            },
            // Read a line from stderr
            Ok(Some(line)) = stderr_reader.next_line() => {
                // Print to local stderr as well
                eprintln!("{}", line);
                send_to_webhook(&http_client, &webhook_url, &line, true).await;
            },
            // The loop breaks when both streams have ended.
            else => break,
        }
    }

    // --- Final Status ---
    let status = child.wait().await?;
    let final_message = match status.code() {
        Some(0) => format!("✅ Success: Command finished with exit code 0."),
        Some(code) => format!("❌ Failure: Command failed with exit code {}.", code),
        None => "❌ Failure: Command was terminated by a signal.".to_string(),
    };

    println!("{}", final_message);
    send_to_webhook(
        &http_client,
        &webhook_url,
        &final_message,
        !status.success(),
    )
    .await;

    // Exit with the same code as the child process.
    if let Some(code) = status.code() {
        std::process::exit(code);
    }

    Ok(())
}