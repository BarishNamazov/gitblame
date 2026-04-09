//! Integration tests for the gitblame CLI binary.

use std::process::Command;

/// Locate the built binary. We rely on `cargo test` having built it.
fn binary_path() -> std::path::PathBuf {
    // cargo puts test binaries in target/debug/
    let mut path = std::env::current_exe()
        .expect("cannot determine test executable path")
        .parent()
        .expect("no parent dir")
        .parent()
        .expect("no grandparent dir")
        .to_path_buf();
    path.push("git");
    path
}

#[test]
fn binary_exists() {
    let bin = binary_path();
    assert!(
        bin.exists(),
        "binary should exist at {}, run `cargo build` first",
        bin.display()
    );
}

#[test]
fn version_passthrough() {
    // `git --version` should be passed through to real git and succeed.
    // We set GITBLAME_REAL_GIT to make sure find_real_git works in CI.
    let real_git = which_git();
    let output = Command::new(binary_path())
        .arg("--version")
        .env("GITBLAME_REAL_GIT", &real_git)
        .output()
        .expect("failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("git version"),
        "expected 'git version' in output, got: {stdout}"
    );
}

#[test]
fn unknown_command_passthrough() {
    // An unknown subcommand should be passed to real git, which will error.
    let real_git = which_git();
    let output = Command::new(binary_path())
        .args(["definitely-not-a-real-command"])
        .env("GITBLAME_REAL_GIT", &real_git)
        .output()
        .expect("failed to execute binary");

    // Real git should exit with non-zero for an unknown subcommand.
    assert!(
        !output.status.success(),
        "unknown subcommand should result in non-zero exit"
    );
}

#[test]
fn blame_with_flags_passthrough() {
    // `git blame --help` should pass through to real git blame.
    let real_git = which_git();
    let output = Command::new(binary_path())
        .args(["blame", "--help"])
        .env("GITBLAME_REAL_GIT", &real_git)
        .output()
        .expect("failed to execute binary");

    // --help always succeeds in real git
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("blame") || output.status.success(),
        "blame --help should pass through to real git"
    );
}

#[test]
fn no_args_passthrough() {
    // Running with no args should pass through to real git (shows help).
    let real_git = which_git();
    let output = Command::new(binary_path())
        .env("GITBLAME_REAL_GIT", &real_git)
        .output()
        .expect("failed to execute binary");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    // git with no args typically prints usage info
    assert!(
        combined.contains("usage") || combined.contains("git"),
        "no-args should show git usage, got: {combined}"
    );
}

/// Find the real git binary on the system.
fn which_git() -> String {
    let output = Command::new("which")
        .arg("git")
        .output()
        .expect("could not run 'which git'");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
