#![allow(dead_code)]
use std::process::{Command, Stdio};

/// Run a command, capturing stdout+stderr. Returns Ok(stdout) on success, Err(stderr) on failure.
pub fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to launch '{}': {}", program, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Run a command that inherits the terminal (for interactive tools).
/// Returns Ok if exit code == 0.
pub fn run_command_inherit(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("Failed to launch '{}': {}", program, e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("'{}' exited with code: {}", program, status))
    }
}
