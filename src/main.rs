use shell_hook::app;

#[tokio::main]
async fn main() {
    let result = app::run().await;

    if let Err(e) = &result {
        eprintln!("[shell_hook] Error: {}", e);
    }

    std::process::exit(result.unwrap_or(1));
}
