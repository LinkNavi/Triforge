use colored::*;
use crate::{api, config::AppConfig};

pub async fn execute(
    query: &str,
    tags: Vec<String>,
    user: Option<String>,
) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let client = api::ApiClient::new(config);
    
    println!("{}", "Searching repositories...".cyan().bold());
    println!();
    
    if !tags.is_empty() {
        println!("{} Tags: {}", "→".blue(), tags.join(", ").yellow());
    }
    if let Some(ref u) = user {
        println!("{} User: {}", "→".blue(), u.yellow());
    }
    println!();
    
    let results = client.search_repos(query, tags, user).await?;
    
    if results.is_empty() {
        println!("{} No repositories found", "→".blue());
        return Ok(());
    }
    
    println!("{} Found {} repositories:", "✓".green(), results.len().to_string().yellow());
    println!();
    
    for (i, repo) in results.iter().enumerate() {
        println!("{} {}", 
            format!("{}.", i + 1).dimmed(),
            repo.name.yellow().bold()
        );
        
        if let Some(desc) = &repo.description {
            println!("   {}", desc);
        }
        
        let short_hash = if repo.repo_hash.len() > 16 { 
            &repo.repo_hash[..16] 
        } else { 
            &repo.repo_hash 
        };
        
        println!("   {} {} | {} ⭐",
            "Hash:".dimmed(),
            short_hash.cyan(),
            repo.star_count.to_string().yellow()
        );
        println!();
    }
    
    Ok(())
}
