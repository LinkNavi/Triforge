use colored::*;
use crate::{api, config::AppConfig};

pub async fn star(hash: &str) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    
    if config.auth_token.is_none() {
        anyhow::bail!("Not authenticated. Run 'triforge login' first.");
    }
    
    let client = api::ApiClient::new(config);
    
    println!("{} Starring repository...", "→".blue());
    client.star_repo(hash).await?;
    
    println!("{} Starred repository: {}", "✓".green(), hash.yellow());
    Ok(())
}

pub async fn unstar(hash: &str) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    
    if config.auth_token.is_none() {
        anyhow::bail!("Not authenticated. Run 'triforge login' first.");
    }
    
    let client = api::ApiClient::new(config);
    
    println!("{} Unstarring repository...", "→".blue());
    client.unstar_repo(hash).await?;
    
    println!("{} Unstarred repository: {}", "✓".green(), hash.yellow());
    Ok(())
}
