use colored::*;
use crate::{api, config::AppConfig};

pub async fn execute(starred: bool, pinned: bool) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    
    if config.auth_token.is_none() {
        println!("{}", "Your Repositories".cyan().bold());
        println!("{}", "═".repeat(60).cyan());
        println!();
        println!("{} This feature requires authentication", "!".yellow());
        println!("{} Run 'triforge login' first", "→".blue());
        println!();
        return Ok(());
    }
    
    let client = api::ApiClient::new(config);
    
    let (repos, title) = if starred {
        println!("{}", "Starred Repositories".cyan().bold());
        (client.get_starred().await?, "starred")
    } else if pinned {
        println!("{}", "Pinned Repositories".cyan().bold());
        (client.get_pinned().await?, "pinned")
    } else {
        println!("{}", "Your Repositories".cyan().bold());
        (client.list_user_repos().await?, "your")
    };
    
    println!("{}", "═".repeat(60).cyan());
    println!();
    
    if repos.is_empty() {
        println!("{} No {} repositories", "→".blue(), title);
        println!();
        println!("{} Create one with:", "→".blue());
        println!("  {}", "triforge init".cyan());
        println!("  {}", "triforge commit -m 'Initial commit'".cyan());
        println!("  {}", "triforge push".cyan());
        println!();
        return Ok(());
    }
    
    println!("{} Found {} repositories:", "✓".green(), repos.len().to_string().yellow());
    println!();
    
    for repo in repos {
        println!("  {} {}", "📦".normal(), repo.name.cyan().bold());
        
        if let Some(desc) = &repo.description {
            println!("     {}", desc.dimmed());
        }
        
        println!("     {} {} | {} KB | {} ⭐ | {} 🍴",
            "Hash:".dimmed(),
            repo.repo_hash[..16].to_string().yellow(),
            (repo.size / 1024).to_string().dimmed(),
            repo.star_count.to_string().yellow(),
            repo.fork_count.to_string().blue()
        );
        
        println!("     {} {} replicas | {}",
            "Health:".dimmed(),
            repo.replica_count.to_string().green(),
            repo.health_status.green()
        );
        
        println!("     {}", format!("triforge clone {}", repo.repo_hash).dimmed());
        println!();
    }
    
    Ok(())
}
