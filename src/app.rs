use crate::cli::Args;
use crate::command::run_command_and_stream;
use crate::error::AppError;
use crate::message::StreamMessage;
use crate::webhook::{run_webhook_sender, send_message};
use clap::Parser;
use reqwest::Client;
use std::io::ErrorKind;
use std::process::ExitStatus;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Shared application context to avoid passing many arguments.
pub struct AppContext {
    pub args: Arc<Args>,
    pub client: Client,
}

/// The main application logic.
pub async fn run() -> Result<i32, AppError> {
    let args = Arc::new(Args::parse());

    // Validate arguments
    if args.webhook_url.is_none() && !args.dry_run {
        return Err(AppError::MissingWebhookUrl);
    }

    let context = Arc::new(AppContext {
        args: args.clone(),
        client: Client::new(),
    });

    // --- Setup communication channel and tasks ---
    let (tx, rx) = mpsc::channel::<StreamMessage>(100);
    let sender_task = tokio::spawn(run_webhook_sender(context.clone(), rx));

    // --- Send initial message ---
    let command_str = args.command.join(" ");
    let title_prefix = if !args.title.is_empty() {
        format!("[{}] ", args.title)
    } else {
        "".to_string()
    };
    let start_message = format!("{}🚀 Starting command: `{}`", title_prefix, command_str);
    println!("{}", start_message);
    send_message(&context, &start_message).await;

    // --- Run command and stream output ---
    let status_result = run_command_and_stream(context.clone(), tx).await;

    // --- Wait for sender to finish sending buffered messages ---
    sender_task.await?;

    // --- Handle command result and send final message ---
    handle_command_result(&context, status_result, &title_prefix).await
}

/// Handles the result of the command execution, sends a final message, and returns the exit code.
async fn handle_command_result(
    context: &Arc<AppContext>,
    status_result: std::io::Result<ExitStatus>,
    title_prefix: &str,
) -> Result<i32, AppError> {
    match status_result {
        Ok(status) => {
            let exit_code = status.code().unwrap_or(1);
            let (base_message, is_error) =
                match status.code() {
                    Some(0) => (
                        context
                            .args
                            .on_success
                            .clone()
                            .unwrap_or_else(|| "✅ Command finished successfully.".to_string()),
                        false,
                    ),
                    Some(code) => (
                        context.args.on_failure.clone().unwrap_or_else(|| {
                            format!("❌ Command failed with exit code {}.", code)
                        }),
                        true,
                    ),
                    None => ("❌ Command was terminated by a signal.".to_string(), true),
                };

            let final_message = format!("{}{}", title_prefix, base_message);
            if is_error {
                eprintln!("{}", final_message);
            } else {
                println!("{}", final_message);
            }
            send_message(context, &final_message).await;
            Ok(exit_code)
        }
        Err(e) => {
            let base_message = context
                .args
                .on_failure
                .clone()
                .unwrap_or_else(|| format!("❌ Command failed to start: {}.", e));
            let final_message = format!("{}{}", title_prefix, base_message);
            eprintln!("{}", final_message);
            send_message(context, &final_message).await;
            // Decide on an exit code for command start failure
            match e.kind() {
                ErrorKind::NotFound => Ok(127),
                _ => Ok(1),
            }
        }
    }
}
