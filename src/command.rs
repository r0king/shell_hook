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

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Spawn tasks to read stdout and stderr concurrently
    let tx_out = tx.clone();
    let quiet_mode = context.args.quiet;
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            println!("{}", line);
            if !quiet_mode {
                if tx_out.send(StreamMessage::Line(line)).await.is_err() {
                    break; // Receiver has been dropped
                }
            }
        }
    });

    let tx_err = tx.clone();
    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            eprintln!("{}", line);
            if !quiet_mode {
                if tx_err.send(StreamMessage::Line(line)).await.is_err() {
                    break; // Receiver has been dropped
                }
            }
        }
    });

    // Wait for the command to complete and for readers to finish
    let status = child.wait().await?;
    let _ = stdout_task.await;
    let _ = stderr_task.await;

    // Signal that the command is done
    let _ = tx.send(StreamMessage::CommandFinished).await;

    Ok(status)
}