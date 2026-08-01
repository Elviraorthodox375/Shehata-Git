use std::process::Command;

#[test]
fn help_exposes_safe_commands_without_force_options() {
    let output = Command::new(env!("CARGO_BIN_EXE_shehata"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for command in [
        "doctor", "accounts", "repos", "status", "test", "push", "mcp",
    ] {
        assert!(stdout.contains(command));
    }
    assert!(!stdout.contains("force-with-lease"));
    assert!(!stdout.contains("force-push"));
}

#[test]
fn json_errors_are_structured_and_stay_on_stdout() {
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shehata"))
        .args(["--json", "status"])
        .arg(temp.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value["error"]["code"].is_string());
    assert!(value["error"]["message"].is_string());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("password="));
}
