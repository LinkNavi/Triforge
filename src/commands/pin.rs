use colored::*;
use crate::{api, config::AppConfig};

pub async fn pin(hash: &str) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    
    if config.auth_token.is_none() {
        anyhow::bail!("Not authenticated. Run 'triforge login' first.");
    }
    
    let client = api::ApiClient::new(config);
    
    println!("{} Pinning repository to profile...", "→".blue());
    client.pin_repo(hash).await?;
    
    println!("{} Pinned repository: {}", "✓".green(), hash.yellow());
    Ok(())
}

pub async fn unpin(hash: &str) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    
    if config.auth_token.is_none() {
        anyhow::bail!("Not authenticated. Run 'triforge login' first.");
    }
    
    let client = api::ApiClient::new(config);
    
    println!("{} Unpinning repository...", "→".blue());
    client.unpin_repo(hash).await?;
    
    println!("{} Unpinned repository: {}", "✓".green(), hash.yellow());
    Ok(())
}
