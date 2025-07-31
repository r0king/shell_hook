use crate::cli::{Cli, Command, RunArgs};
use crate::command::run_command_and_stream;
use crate::error::AppError;
use crate::message::StreamMessage;
use crate::webhook::{run_webhook_sender, send_message};
use clap::Parser;
use dirs::home_dir;
use reqwest::Client;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::io::ErrorKind;
use std::process::ExitStatus;
use std::sync::Arc;
use tokio::sync::mpsc;

const CHANNEL_BUFFER_SIZE: usize = 100;
const HISTORY_FILE: &str = ".shell_hook_history";

/// Shared application context to avoid passing many arguments.
pub struct AppContext {
    pub cli: Arc<Cli>,
    pub client: Client,
}

/// The main application logic.
pub async fn run() -> Result<i32, AppError> {
    let cli = Arc::new(Cli::parse());

    // Validate arguments
    if cli.webhook_url.is_none() && !cli.dry_run {
        return Err(AppError::MissingWebhookUrl);
    }

    let context = Arc::new(AppContext {
        cli: cli.clone(),
        client: Client::new(),
    });

    match &cli.command {
        Command::Run(run_args) => run_single_command(&context, run_args).await,
        Command::Shell => run_shell_session(&context).await,
    }
}

async fn run_single_command(
    context: &Arc<AppContext>,
    run_args: &RunArgs,
) -> Result<i32, AppError> {
    // --- Setup communication channel and tasks ---
    let (tx, rx) = mpsc::channel::<StreamMessage>(CHANNEL_BUFFER_SIZE);
    let sender_task = tokio::spawn(run_webhook_sender(context.clone(), rx));

    // --- Send initial message ---
    let command_str = run_args.command.join(" ");
    let start_message = format_with_title(
        &context.cli,
        &format!("🚀 Starting command: `{}`", command_str),
    );
    println!("{}", start_message);
    if let Err(e) = send_message(context, &start_message).await {
        eprintln!("[shell_hook] Warning: Failed to send start message: {}", e);
    }

    // --- Run command and stream output ---
    let status_result = run_command_and_stream(context.clone(), tx, run_args).await;

    // --- Wait for sender to finish sending buffered messages ---
    sender_task.await?;

    // --- Handle command result and send final message ---
    handle_command_result(context, status_result, run_args).await
}

async fn run_shell_session(context: &Arc<AppContext>) -> Result<i32, AppError> {
    println!("Starting interactive shell session. Type 'exit' to quit.");
    let mut rl = DefaultEditor::new()?;

    let history_path = home_dir().map(|p| p.join(HISTORY_FILE));
    if let Some(ref path) = history_path {
        if let Err(e) = rl.load_history(path) {
            eprintln!("[shell_hook] Warning: Could not load history file: {}", e);
        }
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if let Err(e) = rl.add_history_entry(line.as_str()) {
                    eprintln!("[shell_hook] Warning: Could not add to history: {}", e);
                }
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "exit" {
                    break;
                }

                let run_args = RunArgs {
                    command: vec![line.to_string()],
                    on_success: None,
                    on_failure: None,
                    quiet: false,
                };

                if let Err(e) = run_single_command(context, &run_args).await {
                    eprintln!("[shell_hook] Error executing command: {}", e);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("(To exit, press Ctrl-D or type \"exit\")");
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("[shell_hook] Readline error: {}", err);
                break;
            }
        }
    }

    if let Some(ref path) = history_path {
        if let Err(e) = rl.save_history(path) {
            eprintln!("[shell_hook] Warning: Could not save history file: {}", e);
        }
    }

    Ok(0)
}

/// Handles the result of the command execution, sends a final message, and returns the exit code.
async fn handle_command_result(
    context: &Arc<AppContext>,
    status_result: std::io::Result<ExitStatus>,
    run_args: &RunArgs,
) -> Result<i32, AppError> {
    match status_result {
        Ok(status) => {
            let exit_code = status.code().unwrap_or(1);
            let (base_message, is_error) = match status.code() {
                Some(0) => (
                    run_args
                        .on_success
                        .clone()
                        .unwrap_or_else(|| "✅ Command finished successfully.".to_string()),
                    false,
                ),
                Some(code) => (
                    run_args
                        .on_failure
                        .clone()
                        .unwrap_or_else(|| format!("❌ Command failed with exit code {}.", code)),
                    true,
                ),
                None => ("❌ Command was terminated by a signal.".to_string(), true),
            };

            let final_message = format_with_title(&context.cli, &base_message);
            if is_error {
                eprintln!("{}", final_message);
            } else {
                println!("{}", final_message);
            }
            if let Err(e) = send_message(context, &final_message).await {
                eprintln!("[shell_hook] Warning: Failed to send final message: {}", e);
            }
            Ok(exit_code)
        }
        Err(e) => {
            let base_message = run_args
                .on_failure
                .clone()
                .unwrap_or_else(|| format!("❌ Command failed to start: {}.", e));
            let final_message = format_with_title(&context.cli, &base_message);
            eprintln!("{}", final_message);
            if let Err(e) = send_message(context, &final_message).await {
                eprintln!(
                    "[shell_hook] Warning: Failed to send failure message: {}",
                    e
                );
            }
            // Decide on an exit code for command start failure
            match e.kind() {
                ErrorKind::NotFound => Ok(127),
                _ => Ok(1),
            }
        }
    }
}

/// Formats a message with the title prefix if a title is provided.
fn format_with_title(cli: &Cli, message: &str) -> String {
    if let Some(title) = &cli.title {
        format!("[{}] {}", title, message)
    } else {
        message.to_string()
    }
}
