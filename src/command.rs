use crate::app::AppContext;
use crate::cli::RunArgs;
use crate::message::StreamMessage;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

/// Spawns the command, captures its stdout/stderr, and sends lines to the channel.
pub async fn run_command_and_stream(
    _context: Arc<AppContext>,
    tx: mpsc::Sender<StreamMessage>,
    run_args: &RunArgs,
) -> std::io::Result<ExitStatus> {
    // For the `run` subcommand, we execute the command directly.
    // For the `shell` subcommand, we wrap the command in `sh -c`.
    // This is now handled in `app.rs` by creating the appropriate command vector.
    let command_str = run_args.command.join(" ");
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&command_str)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut tasks = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        tasks.push(stream_output(stdout, tx.clone(), run_args.quiet, false));
    }
    if let Some(stderr) = child.stderr.take() {
        tasks.push(stream_output(stderr, tx.clone(), run_args.quiet, true));
    }

    // Wait for the command to complete and for readers to finish
    let status = child.wait().await?;
    for task in tasks {
        let _ = task.await;
    }

    // Signal that the command is done
    let _ = tx.send(StreamMessage::CommandFinished).await;

    Ok(status)
}

/// Helper to stream output from a reader to a channel, printing lines to stdout/stderr.
fn stream_output<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    reader: R,
    tx: mpsc::Sender<StreamMessage>,
    quiet_mode: bool,
    is_stderr: bool,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut reader = BufReader::new(reader).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if is_stderr {
                eprintln!("{}", line);
            } else {
                println!("{}", line);
            }
            if !quiet_mode && tx.send(StreamMessage::Line(line)).await.is_err() {
                break; // Receiver has been dropped
            }
        }
    })
}
