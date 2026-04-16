use crate::models::tool::Tool;

/// Brew (Homebrew) provider.
/// Brew is macOS/Linux only. On Windows this provider returns an empty set.
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
        struct BrewSearchResult {
            formulae: Vec<BFormula>,
        }

        #[derive(Deserialize)]
        struct BFormula {
            name: String,
            desc: Option<String>,
            versions: BVersions,
        }

        #[derive(Deserialize)]
        struct BVersions {
            stable: Option<String>,
        }

        let url = format!("https://formulae.brew.sh/api/formula.json");
        let res = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("climart/0.1.0")
            .build()?
            .get(&url)
            .send()
            .await;

        let formulae: Vec<BFormula> = match res {
            Ok(r) if r.status().is_success() => r.json().await?,
            _ => return Ok(vec![]),
        };

        let q = query.to_lowercase();
        let tools: Vec<Tool> = formulae
            .into_iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&q)
                    || f.desc
                        .as_deref()
                        .map(|d| d.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .take(10)
            .map(|f| {
                let version = f.versions.stable.unwrap_or_else(|| "latest".to_string());
                Tool {
                    install_command: format!("brew install {}", f.name),
                    run_command: f.name.clone(),
                    binary: Some(f.name.clone()),
                    name: f.name.clone(),
                    description: f.desc.unwrap_or_default(),
                    source: "brew".to_string(),
                    version,
                }
            })
            .collect();

        Ok(tools)
    }
}
