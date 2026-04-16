use crate::models::tool::Tool;

/// pkgx provider.
/// pkgx is primarily macOS/Linux. On Windows this provider returns an empty set.
pub async fn search(query: &str, _size: usize) -> Result<Vec<Tool>, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(target_os = "windows")]
    {
        let _ = query;
        return Ok(vec![]);
    }

    #[cfg(not(target_os = "windows"))]
    {
        use reqwest::Client;
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct PkgxPackage {
            project: String,
            description: Option<String>,
            version: Option<String>,
        }

        // pkgx has a package index at their pantry
        let url = "https://pkgx.dev/pkgs/";
        let res = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("climart/0.1.0")
            .build()?
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await;

        // pkgx doesn't have a clean search API — do a best-guess name match
        let q = query.to_lowercase();
        match res {
            Ok(r) if r.status().is_success() => {
                // Try JSON list
                let pkgs: Result<Vec<PkgxPackage>, _> = r.json().await;
                match pkgs {
                    Ok(list) => {
                        let tools: Vec<Tool> = list
                            .into_iter()
                            .filter(|p| p.project.to_lowercase().contains(&q))
                            .take(5)
                            .map(|p| {
                                let name = p.project.clone();
                                let version = p.version.unwrap_or_else(|| "latest".to_string());
                                Tool {
                                    install_command: format!("pkgx install {}", name),
                                    run_command: name.clone(),
                                    binary: Some(name.clone()),
                                    name,
                                    description: p.description.unwrap_or_default(),
                                    source: "pkgx".to_string(),
                                    version,
                                }
                            })
                            .collect();
                        Ok(tools)
                    }
                    Err(_) => Ok(vec![]),
                }
            }
            _ => {
                // Fallback: return a simple "try this with pkgx" stub if query matches a common tool
                let common = ["node", "python", "ruby", "go", "rust", "deno", "bun"];
                if common.iter().any(|&c| c == q) {
                    Ok(vec![Tool {
                        name: query.to_string(),
                        description: format!("Install {} via pkgx", query),
                        source: "pkgx".to_string(),
                        version: "latest".to_string(),
                        install_command: format!("pkgx install {}", query),
                        run_command: format!("pkgx {}", query),
                        binary: Some(query.to_string()),
                    }])
                } else {
                    Ok(vec![])
                }
            }
        }
    }
}
