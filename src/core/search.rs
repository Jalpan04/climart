use crate::models::tool::Tool;
use crate::providers::{npm, pipx, brew, pkgx};
use crate::config::settings::Config;

pub type SearchError = Box<dyn std::error::Error + Send + Sync>;

pub async fn search(query: &str, config: &Config) -> Result<Vec<Tool>, SearchError> {
    let size = config.ui.results_per_page;

    // Run enabled providers concurrently
    let npm_fut = async {
        if config.providers.npm {
            npm::search(query, size).await.unwrap_or_default()
        } else {
            vec![]
        }
    };

    let pipx_fut = async {
        if config.providers.pipx {
            pipx::search(query, size).await.unwrap_or_default()
        } else {
            vec![]
        }
    };

    let brew_fut = async {
        if config.providers.brew {
            brew::search(query, size).await.unwrap_or_default()
        } else {
            vec![]
        }
    };

    let pkgx_fut = async {
        if config.providers.pkgx {
            pkgx::search(query, size).await.unwrap_or_default()
        } else {
            vec![]
        }
    };

    let (npm_res, pipx_res, brew_res, pkgx_res) =
        tokio::join!(npm_fut, pipx_fut, brew_fut, pkgx_fut);

    let mut tools: Vec<Tool> = npm_res
        .into_iter()
        .chain(pipx_res)
        .chain(brew_res)
        .chain(pkgx_res)
        .collect();

    // Filter: scoped npm packages must have "cli" in name/description
    tools.retain(|tool| {
        if tool.source == "npm" && tool.name.starts_with('@') {
            let q = query.to_lowercase();
            tool.name.to_lowercase().contains(&q)
                || tool.description.to_lowercase().contains("cli")
                || tool.name.to_lowercase().contains("cli")
        } else {
            true
        }
    });

    // De-duplicate by name (keep first occurrence)
    let mut seen = std::collections::HashSet::new();
    tools.retain(|t| seen.insert(t.name.clone()));

    // Fill in missing binary names
    for tool in &mut tools {
        if tool.binary.is_none() {
            let bin = tool.name.split('/').last().unwrap_or(&tool.name);
            tool.binary = Some(bin.to_string());
        }
    }

    Ok(tools)
}
