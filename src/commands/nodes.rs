use colored::*;
use crate::{api, config::AppConfig};

pub async fn execute() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let client = api::ApiClient::new(config);
    
    println!("{}", "Active Storage Nodes".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    
    let nodes = client.list_nodes().await?;
    
    if nodes.is_empty() {
        println!("{} No active nodes", "→".blue());
        return Ok(());
    }
    
    println!("{} Found {} active nodes:", "✓".green(), nodes.len().to_string().yellow());
    println!();
    
    for (i, node) in nodes.iter().enumerate() {
        let node_type = if node.is_anchor {
            "⚓ Anchor".yellow()
        } else {
            "🌐 P2P".blue()
        };
        
        println!("{} {}", 
            format!("{}.", i + 1).dimmed(),
            node_type
        );
        println!("   Address: {}:{}", node.address, node.port);
        println!("   ID: {}", node.node_id[..16].to_string().dimmed());
        println!("   Capacity: {} GB", (node.storage_capacity / 1_000_000_000).to_string().cyan());
        println!();
    }
    
    Ok(())
}
