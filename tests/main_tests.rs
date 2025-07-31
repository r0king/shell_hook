use std::process::Command;

#[test]
fn test_main_binary_success() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "shell_hook", "--", "--webhook-url", "http://localhost", "run", "--", "echo", "hello"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
}

#[test]
fn test_main_binary_error() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "shell_hook", "--", "run"])
        .output()
        .expect("failed to execute process");

    assert!(!output.status.success());
}
