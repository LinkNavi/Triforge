// src/commands/info.rs
use colored::*;
use crate::{api, config::AppConfig};

pub async fn execute(hash: &str) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let client = api::ApiClient::new(config);
    
    println!("{}", "Fetching repository information...".cyan());
    println!();
    
    let repo_hash = hash.trim_start_matches("hyrule://").to_string();
    let metadata = client.get_repo(&repo_hash).await?;
    
    // Header
    println!("{}", "═".repeat(60).cyan());
    println!("{}", metadata.name.yellow().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    
    // Basic info
    if let Some(desc) = &metadata.description {
        println!("{}", desc);
        println!();
    }
    
    println!("{} {}", "Hash:".bold(), metadata.repo_hash.green());
    println!("{} {} KB", "Size:".bold(), (metadata.size / 1024).to_string().yellow());
    println!();
    
    // Network status
    println!("{}", "Network Status".bold().underline());
    println!("{} {}", "Replicas:".bold(), metadata.replica_count.to_string().cyan());
    println!("{} {}", "Health:".bold(), metadata.health_status.green());
    println!();
    
    // Nodes
    if !metadata.nodes.is_empty() {
        println!("{}", "Storage Nodes".bold().underline());
        for (i, node) in metadata.nodes.iter().enumerate() {
            let node_type = if node.is_anchor { 
                "⚓ Anchor".yellow() 
            } else { 
                "🌐 P2P".blue() 
            };
            
            println!("{}. {} {}", 
                (i + 1).to_string().cyan(),
                node_type,
                format!("{}:{}", node.address, node.port).dimmed()
            );
            println!("   ID: {}", node.node_id[..16].to_string().dimmed());
        }
        println!();
    }
    
    // Clone command
    println!("{}", "Clone this repository:".bold());
    println!("  {}", format!("triforge clone {}", metadata.repo_hash).cyan());
    println!();
    
    Ok(())
}
