use hook_stream::app;

#[tokio::main]
async fn main() {
    let result = app::run().await;

    if let Err(e) = &result {
        eprintln!("[hook-stream] Error: {}", e);
    }

    std::process::exit(match result {
        Ok(code) => code,
        Err(_) => 1,
    });
}
