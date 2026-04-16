use reqwest::Client;
use serde::Deserialize;
use crate::models::tool::Tool;

#[derive(Deserialize)]
struct SearchResponse {
    objects: Vec<SearchObject>,
}

#[derive(Deserialize)]
struct SearchObject {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    description: Option<String>,
    version: Option<String>,
    keywords: Option<Vec<String>>,
}

pub async fn search(query: &str, size: usize) -> Result<Vec<Tool>, reqwest::Error> {
    let url = format!(
        "https://registry.npmjs.org/-/v1/search?text={}&size={}",
        urlencoding(query),
        size
    );

    let response: SearchResponse = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("climart/0.1.0")
        .build()?
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    let tools = response.objects.into_iter().map(|obj| {
        let name = obj.package.name;
        let description = obj.package.description.unwrap_or_default();
        let version = obj.package.version.unwrap_or_else(|| "latest".to_string());
        let _keywords = obj.package.keywords.unwrap_or_default();

        let install_command = format!("npm install -g {}", name);
        let run_command = if cfg!(target_os = "windows") {
            format!("npx.cmd {}", name)
        } else {
            format!("npx {}", name)
        };

        let binary = Some(name.split('/').last().unwrap_or(&name).to_string());

        Tool {
            name,
            description,
            source: "npm".to_string(),
            version,
            binary,
            install_command,
            run_command,
        }
    }).collect();

    Ok(tools)
}

// Simple percent-encoding for query strings
fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            other => format!("%{:02X}", other as u32),
        })
        .collect()
}
