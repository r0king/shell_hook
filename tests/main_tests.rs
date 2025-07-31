use shell_hook::app;

#[tokio::test]
async fn test_run_app_success() {
    let result = app::run_from(vec!["shell_hook", "--dry-run", "run", "echo", "hello"]).await;
    assert_eq!(result.unwrap(), 0);
}

#[tokio::test]
async fn test_run_app_error() {
    let result = app::run_from(vec!["shell_hook", "run"]).await;
    assert!(result.is_err());
}