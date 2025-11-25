use colored::*;
use crate::{api, config::AppConfig};

pub async fn execute(
    hash: &str,
    name: Option<String>,
    description: Option<String>,
) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    
    if config.auth_token.is_none() {
        anyhow::bail!("Not authenticated. Run 'triforge login' first.");
    }
    
    let client = api::ApiClient::new(config);
    
    println!("{}", "Forking repository...".cyan().bold());
    println!();
    
    let repo_hash = hash.trim_start_matches("hyrule://").to_string();
    
    // Get original repo metadata
    let original = client.get_repo(&repo_hash).await?;
    
    let fork_name = name.unwrap_or_else(|| format!("{}-fork", original.name));
    let fork_desc = description.or(original.description.clone());
    
    println!("{} Original: {}", "→".blue(), original.name.yellow());
    println!("{} Fork name: {}", "→".blue(), fork_name.yellow());
    
    // Create fork
    let response = client.fork_repo(&repo_hash, &fork_name, fork_desc.as_deref()).await?;
    
    println!();
    println!("{} Successfully forked repository!", "✓".green().bold());
    println!("{} Fork hash: {}", "→".blue(), response.forked_hash.green());
    println!();
    println!("{} Clone your fork with:", "→".blue());
    println!("  {}", format!("triforge clone {}", response.forked_hash).cyan());
    println!();
    
    Ok(())
}
