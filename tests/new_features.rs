use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use assert_fs::fixture::{PathChild, FileWriteStr};

/// Builds the example binary and returns a Command prepared to run it.
fn get_example_cmd() -> Command {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let mut cmd = Command::new("cargo");
        cmd.args(["build", "--example", "showcase"]);
        let status = cmd.status().expect("Failed to build example");
        if !status.success() {
            panic!("Failed to build example");
        }
    });

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let binary_path = std::path::Path::new(&manifest_dir)
        .join("target/debug/examples/showcase");

    Command::new(binary_path)
}

#[test]
fn test_date_and_benchmark_macros() {
    let mut cmd = get_example_cmd();
    cmd.env("DEBUG", "1").arg("date-test");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Epoch:"))
        .stderr(predicate::str::contains("Benchmark completed in"));
}

#[test]
fn test_file_in_macro() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child("file1.txt").write_str("content1").unwrap();
    temp_dir.child("file2.txt").write_str("content2").unwrap();

    let mut cmd = get_example_cmd();
    cmd.arg("file-in-test").arg(temp_dir.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"));
}

#[test]
fn test_path_split_macro() {
    let mut cmd = get_example_cmd();
    cmd.arg("path-test").arg("/tmp/some/file.txt");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Parent: /tmp/some"))
        .stdout(predicate::str::contains("Filename: file.txt"));
}

#[test]
fn test_job_control_wait_and_timeout() {
    // Test successful wait
    let mut wait_cmd = get_example_cmd();
    wait_cmd.arg("job-test-integration");
    wait_cmd.assert()
        .success()
        .stdout(predicate::str::contains("wait_status=0"));

    // Test timeout
    let mut timeout_cmd = get_example_cmd();
    timeout_cmd.arg("job-test-timeout-integration");
    timeout_cmd.assert()
        .success()
        .stderr(predicate::str::contains("Timeout"))
        .stdout(predicate::str::contains("timeout_status=-1"));
}

#[test]
fn test_sed_block() {
    let mut cmd = get_example_cmd();
    cmd.arg("sed-block-test");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<setting>new_value</setting>"))
        .stdout(predicate::str::contains("Unclosed block contains: old_value"));
}

#[test]
fn test_color_config() {
    let mut cmd = get_example_cmd();
    // Override the color for 'error' and the glyph for 'warn'
    cmd.env("RSB_COLORS", "error:[magenta],warn:[,>>]");
    cmd.env("DEBUG", "1"); // Ensure all levels are printed
    cmd.arg("color-test");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("\x1b[35m")) // Check for magenta color code for error
        .stderr(predicate::str::contains(">> This is a warning message.")); // Check for new warn glyph
}
