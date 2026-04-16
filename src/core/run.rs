use std::process::Command;
use crate::models::tool::Tool;

pub fn run(tool: &Tool) -> Result<(), String> {
    let candidates = build_candidates(tool);

    let mut last_err = String::from("No execution candidate available");
    for (cmd_name, args) in candidates {
        match try_run(&cmd_name, &args) {
            Ok(()) => return Ok(()),
            Err(e) if e.contains("not found") || e.contains("No such file") => {
                last_err = e;
                continue; // try next
            }
            Err(e) => return Err(e), // hard failure
        }
    }

    Err(format!("All run attempts failed. Last error: {}", last_err))
}

// Build an ordered list of (command, args) to try
fn build_candidates(tool: &Tool) -> Vec<(String, Vec<String>)> {
    let mut candidates: Vec<(String, Vec<String>)> = Vec::new();

    // 1. Use the explicit run_command if provided
    if !tool.run_command.is_empty() {
        let mut parts = tool.run_command.split_whitespace();
        if let Some(cmd) = parts.next() {
            let cmd = resolve_cmd(tool, cmd);
            let args: Vec<String> = parts.map(String::from).collect();
            candidates.push((cmd, args));
        }
    }

    // 2. Binary field
    if let Some(bin) = &tool.binary {
        if !bin.is_empty() {
            let cmd = resolve_cmd(tool, bin);
            candidates.push((cmd, vec![]));
        }
    }

    // 3. Provider-specific fallback
    match tool.source.as_str() {
        "npm" => {
            let npx = if cfg!(target_os = "windows") { "npx.cmd" } else { "npx" };
            candidates.push((npx.to_string(), vec![tool.name.clone()]));
        }
        "pipx" => {
            candidates.push(("pipx".to_string(), vec!["run".to_string(), tool.name.clone()]));
        }
        "pkgx" => {
            candidates.push(("pkgx".to_string(), vec![tool.name.clone()]));
        }
        _ => {
            candidates.push((tool.name.clone(), vec![]));
        }
    }

    candidates
}

fn resolve_cmd(tool: &Tool, cmd: &str) -> String {
    if cfg!(target_os = "windows") && (tool.source == "npm") {
        if cmd == "npx" || cmd == "npm" {
            return format!("{}.cmd", cmd);
        }
    }
    cmd.to_string()
}

fn try_run(cmd_name: &str, args: &[String]) -> Result<(), String> {
    let mut cmd = Command::new(cmd_name);
    cmd.args(args);

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(format!("'{}' exited with code: {}", cmd_name, status))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Err(format!("Command '{}' not found", cmd_name))
            } else {
                Err(format!("Failed to execute '{}': {}", cmd_name, e))
            }
        }
    }
}
