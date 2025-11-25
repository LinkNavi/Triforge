use colored::*;
use std::io::{self, Write};
use crate::{api, config::AppConfig};

pub async fn execute(hash: &str, force: bool) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    
    if config.auth_token.is_none() {
        anyhow::bail!("Not authenticated. Run 'triforge login' first.");
    }
    
    let client = api::ApiClient::new(config);
    
    // Get repo info first
    let repo = client.get_repo(hash).await?;
    
    println!("{}", "Delete Repository".red().bold());
    println!("{}", "═".repeat(60).red());
    println!();
    println!("{} Repository: {}", "→".blue(), repo.name.yellow());
    println!("{} Hash: {}", "→".blue(), hash.dimmed());
    println!();
    println!("{} This action cannot be undone!", "⚠".red().bold());
    println!();
    
    if !force {
        print!("{} ", "Type the repository name to confirm:".yellow());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim() != repo.name {
            println!("{} Deletion cancelled", "✓".green());
            return Ok(());
        }
    }
    
    println!("{} Deleting repository...", "→".blue());
    client.delete_repo(hash).await?;
    
    println!();
    println!("{} Repository deleted", "✓".green());
    
    Ok(())
}
