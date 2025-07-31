use shell_hook::app;
use shell_hook::error::AppError;

#[tokio::main]
async fn main() {
    let result = run_app().await;

    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("[shell_hook] Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_app() -> Result<i32, AppError> {
    app::run().await
}
