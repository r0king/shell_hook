use crate::app::AppContext;
use crate::message::StreamMessage;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

/// Spawns the command, captures its stdout/stderr, and sends lines to the channel.
pub async fn run_command_and_stream(
    context: Arc<AppContext>,
    tx: mpsc::Sender<StreamMessage>,
) -> std::io::Result<ExitStatus> {
    let mut child = Command::new(&context.args.command[0])
        .args(&context.args.command[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut tasks = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        tasks.push(stream_output(stdout, tx.clone(), context.args.quiet, false));
    }
    if let Some(stderr) = child.stderr.take() {
        tasks.push(stream_output(stderr, tx.clone(), context.args.quiet, true));
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
