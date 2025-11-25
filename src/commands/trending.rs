use colored::*;
use crate::{api, config::AppConfig};

pub async fn execute(limit: usize) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let client = api::ApiClient::new(config);
    
    println!("{}", "Trending Repositories".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    
    let repos = client.get_trending(limit).await?;
    
    if repos.is_empty() {
        println!("{} No trending repositories", "→".blue());
        return Ok(());
    }
    
    for (i, repo) in repos.iter().enumerate() {
        println!("{} {}", 
            format!("{}.", i + 1).yellow().bold(),
            repo.name.cyan().bold()
        );
        
        if let Some(desc) = &repo.description {
            println!("   {}", desc.dimmed());
        }
        
        println!("   {} ⭐  {} 🍴  {} KB",
            repo.star_count.to_string().yellow(),
            repo.fork_count.to_string().blue(),
            (repo.size / 1024).to_string().dimmed()
        );
        
        println!("   {}", format!("triforge clone {}", repo.repo_hash).dimmed());
        println!();
    }
    
    Ok(())
}
