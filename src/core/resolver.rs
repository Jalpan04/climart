#![allow(dead_code)]
use std::process::Command;

/// Returns true if a command exists on PATH
pub fn command_exists(cmd: &str) -> bool {
    let (prog, arg) = if cfg!(target_os = "windows") {
        ("where", cmd)
    } else {
        ("which", cmd)
    };
    Command::new(prog)
        .arg(arg)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns the list of provider names that are available on this machine
pub fn available_providers() -> Vec<&'static str> {
    let mut providers = Vec::new();
    if command_exists("npm") {
        providers.push("npm");
    }
    if command_exists("pipx") {
        providers.push("pipx");
    }
    if command_exists("brew") {
        providers.push("brew");
    }
    if command_exists("pkgx") {
        providers.push("pkgx");
    }
    providers
}

/// Maps a source string to the primary install command verb
pub fn install_cmd_for(source: &str, package_name: &str) -> String {
    match source {
        "npm"  => format!("npm install -g {}", package_name),
        "pipx" => format!("pipx install {}", package_name),
        "brew" => format!("brew install {}", package_name),
        "pkgx" => format!("pkgx install {}", package_name),
        other  => format!("echo 'No installer for source: {}'", other),
    }
}

/// Maps a source string to the primary run command
pub fn run_cmd_for(source: &str, package_name: &str) -> String {
    match source {
        "npm"  => {
            if cfg!(target_os = "windows") {
                format!("npx.cmd {}", package_name)
            } else {
                format!("npx {}", package_name)
            }
        }
        "pipx" => format!("pipx run {}", package_name),
        "brew" => package_name.to_string(),
        "pkgx" => format!("pkgx {}", package_name),
        _      => package_name.to_string(),
    }
}
