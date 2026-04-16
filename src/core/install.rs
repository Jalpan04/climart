use std::process::Command;
use crate::models::tool::Tool;

pub fn install(tool: &Tool) -> Result<(), String> {
    let (prog, args): (&str, Vec<&str>) = match tool.source.as_str() {
        "npm" => {
            let cmd = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };
            (cmd, vec!["install", "-g", &tool.name])
        }
        "pipx" => ("pipx", vec!["install", &tool.name]),
        "brew" => ("brew", vec!["install", &tool.name]),
        "pkgx" => ("pkgx", vec!["install", &tool.name]),
        other => return Err(format!("Unsupported source: {}", other)),
    };

    let status = Command::new(prog)
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to launch '{}': {}", prog, e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{} install failed with: {}", tool.source, status))
    }
}
