//! Integration tests for busmaster-cli

use std::path::PathBuf;
use std::process::Command;

fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up to crates
    path.pop(); // Go up to workspace root
    path.push("target");
    path.push("debug");
    path.push("busmaster");
    path
}

#[test]
fn test_cli_help() {
    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BUSMASTER"));
    assert!(stdout.contains("monitor"));
    assert!(stdout.contains("send"));
    assert!(stdout.contains("list"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("busmaster"));
}

#[test]
fn test_list_command() {
    let output = Command::new(get_binary_path())
        .arg("list")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("stub"));
    assert!(stdout.contains("Virtual CAN device"));
}

#[test]
fn test_monitor_help() {
    let output = Command::new(get_binary_path())
        .arg("monitor")
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Monitor CAN bus traffic"));
    assert!(stdout.contains("--driver"));
    assert!(stdout.contains("--dbc"));
    assert!(stdout.contains("--log"));
    assert!(stdout.contains("--filter-range"));
    assert!(stdout.contains("--filter-ids"));
}

#[test]
fn test_send_help() {
    let output = Command::new(get_binary_path())
        .arg("send")
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Send a CAN message"));
    assert!(stdout.contains("--id"));
    assert!(stdout.contains("--data"));
    assert!(stdout.contains("--extended"));
}

#[test]
fn test_invalid_command() {
    let output = Command::new(get_binary_path())
        .arg("invalid")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
}

#[test]
fn test_send_missing_id() {
    let output = Command::new(get_binary_path())
        .arg("send")
        .arg("--data")
        .arg("01 02 03")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--id") || stderr.contains("required"));
}

#[test]
fn test_send_missing_data() {
    let output = Command::new(get_binary_path())
        .arg("send")
        .arg("--id")
        .arg("0x123")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--data") || stderr.contains("required"));
}
