use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: ProvidersConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub npm: bool,
    pub pipx: bool,
    pub brew: bool,
    pub pkgx: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub results_per_page: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            providers: ProvidersConfig {
                npm: true,
                pipx: false,
                brew: false,
                pkgx: false,
            },
            ui: UiConfig {
                results_per_page: 20,
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if let Some(p) = path {
            if p.exists() {
                if let Ok(content) = fs::read_to_string(&p) {
                    if let Ok(cfg) = toml::from_str::<Config>(&content) {
                        return cfg;
                    }
                }
            } else {
                // Write the defaults so users have a reference file
                let _ = Self::save_defaults(&p);
            }
        }
        Config::default()
    }

    fn save_defaults(path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = r#"# Climart configuration
# ~/.climart/config.toml

[providers]
npm   = true
pipx  = false
brew  = false
pkgx  = false

[ui]
results_per_page = 20
"#;
        fs::write(path, content)
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".climart").join("config.toml"))
}
