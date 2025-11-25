use colored::*;
use crate::{api, config::AppConfig};

pub async fn execute() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let client = api::ApiClient::new(config);
    
    println!("{}", "Hyrule Network Statistics".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    
    let stats = client.get_network_stats().await?;
    
    println!("{} Total Repositories: {}", "📦".normal(), stats.total_repos.to_string().yellow().bold());
    println!("{} Storage Nodes: {}", "🌐".normal(), stats.total_nodes.to_string().cyan().bold());
    println!("{} Active Users: {}", "👥".normal(), stats.total_users.to_string().green().bold());
    println!("{} Total Storage: {} GB", "💾".normal(), (stats.total_storage / 1_000_000_000).to_string().blue().bold());
    println!();
    
    println!("{}", "Network Health".bold());
    println!("  Anchor Nodes: {}", stats.anchor_nodes.to_string().green());
    println!("  P2P Nodes: {}", stats.p2p_nodes.to_string().blue());
    println!("  Average Replication: {}x", stats.avg_replication.to_string().yellow());
    println!();
    
    Ok(())
}
