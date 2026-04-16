use reqwest::Client;
use serde::Deserialize;
use crate::models::tool::Tool;

// PyPI JSON search API (uses the simple search endpoint)
#[derive(Deserialize)]
struct PypiSearchResult {
    info: PackageInfo,
}

#[derive(Deserialize)]
struct PackageInfo {
    name: String,
    summary: Option<String>,
    version: String,
}

/// Search PyPI for CLI-related packages matching the query.
/// PyPI doesn't have a proper search API, so we look up the package by name directly.
pub async fn search(query: &str, _size: usize) -> Result<Vec<Tool>, Box<dyn std::error::Error + Send + Sync>> {
    // Try a direct lookup first
    let url = format!("https://pypi.org/pypi/{}/json", query.trim());

    let res = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("climart/0.1.0")
        .build()?
        .get(&url)
        .send()
        .await;

    match res {
        Ok(response) if response.status().is_success() => {
            let pkg: PypiSearchResult = response.json().await?;
            let name = pkg.info.name;
            let description = pkg.info.summary.unwrap_or_default();
            let version = pkg.info.version;

            let tool = Tool {
                install_command: format!("pipx install {}", name),
                run_command: name.clone(),
                binary: Some(name.clone()),
                name,
                description,
                source: "pipx".to_string(),
                version,
            };
            Ok(vec![tool])
        }
        // No match or network error — return empty (caller handles it)
        _ => Ok(vec![]),
    }
}
