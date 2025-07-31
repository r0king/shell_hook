use shell_hook::app;

#[tokio::main]
async fn main() {
    let result = app::run().await;

    if let Err(e) = &result {
        eprintln!("[shell_hook] Error: {}", e);
    }

    std::process::exit(match result {
        Ok(code) => code,
        Err(_) => 1,
    });
}
