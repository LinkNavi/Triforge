// TriForge/src/commands/tags.rs - Complete implementation
use colored::*;
use crate::{api, config::AppConfig};

pub async fn add(hash: &str, tags: Vec<String>) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    
    if config.auth_token.is_none() {
        anyhow::bail!("Not authenticated. Run 'triforge login' first.");
    }
    
    let client = api::ApiClient::new(config);
    
    println!("{} Adding tags to repository...", "→".blue());
    client.add_tags(hash, tags.clone()).await?;
    
    println!("{} Added tags: {}", "✓".green(), tags.join(", ").yellow());
    Ok(())
}

pub async fn list(hash: Option<String>) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let client = api::ApiClient::new(config);
    
    if let Some(h) = hash {
        // List tags for specific repository
        let display_hash = if h.len() > 16 { &h[..16] } else { &h };
        println!("{} Tags for {}:", "→".blue(), display_hash.yellow());
        let tags = client.get_repo_tags(&h).await?;
        
        if tags.is_empty() {
            println!("   No tags");
        } else {
            for tag in tags {
                println!("   • {}", tag.cyan());
            }
        }
    } else {
        // List all available tags
        println!("{}", "All Tags".cyan().bold());
        println!("{}", "═".repeat(60).cyan());
        println!();
        
        let tags = client.get_all_tags().await?;
        
        for (tag, count) in tags {
            println!("  {} ({})", tag.yellow(), count.to_string().dimmed());
        }
    }
    
    Ok(())
}

pub async fn search(tag: &str) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let client = api::ApiClient::new(config);
    
    println!("{} Repositories tagged with '{}':", "→".blue(), tag.yellow());
    println!();
    
    let repos = client.get_repos_by_tag(tag).await?;
    
    if repos.is_empty() {
        println!("   No repositories found");
        return Ok(());
    }
    
    for repo in repos {
        println!("  • {}", repo.name.cyan());
        println!("    {}", repo.repo_hash[..16].to_string().dimmed());
        println!();
    }
    
    Ok(())
}
