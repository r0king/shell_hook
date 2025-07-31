use shell_hook::app;

#[tokio::main]
async fn main() {
    let result = app::run().await;

    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("[shell_hook] Error: {}", e);
            std::process::exit(1);
        }
    }
}
