// src/commands/config.rs
use colored::*;
use crate::config::AppConfig;

pub fn set(key: &str, value: &str) -> anyhow::Result<()> {
    let mut config = AppConfig::load()?;
    config.set(key, value)?;
    config.save()?;
    
    println!("{} Set {} = {}", "✓".green(), key.yellow(), value.cyan());
    Ok(())
}

pub fn get(key: &str) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    
    if let Some(value) = config.get(key) {
        println!("{}", value.cyan());
    } else {
        println!("{} Key not found: {}", "✗".red(), key);
    }
    
    Ok(())
}

pub fn show() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let config_path = AppConfig::config_path()?;
    
    println!("{}", "Configuration".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    println!("{} {}", "AppConfig file:".bold(), config_path.display().to_string().dimmed());
    println!();
    
    println!("{}", "Server Configuration".bold().underline());
    println!("{} {}", "hyrule_server:".yellow(), config.hyrule_server.cyan());
    println!("{} {}", "username:".yellow(), 
        config.username.as_deref().unwrap_or("(not set)").cyan());
    println!("{} {}", "auth_token:".yellow(), 
        if config.auth_token.is_some() { "********".cyan() } else { "(not set)".dimmed() });
    println!();
    
    println!("{}", "Privacy Configuration".bold().underline());
    println!("{} {}", "default_private:".yellow(), config.default_private.to_string().cyan());
    println!();
    
    println!("{}", "Tor Configuration".bold().underline());
    println!("{} {}", "use_tor:".yellow(), 
        if config.use_tor { 
            "enabled".green().bold() 
        } else { 
            "disabled".red() 
        }
    );
    println!("{} {}", "tor_proxy:".yellow(), config.tor_proxy.cyan());
    
    if config.use_tor {
        print!("{} ", "tor_status:".yellow());
        if config.check_tor_available() {
            println!("{}", "✓ connected".green());
        } else {
            println!("{}", "✗ not available".red().bold());
            println!();
            println!("{}", "⚠ Warning: Tor is enabled but not reachable!".yellow().bold());
            println!("{}", "→ Make sure Tor is running:".blue());
            println!("   • Linux/Mac: {}", "sudo systemctl start tor".cyan());
            println!("   • Or install: {}", "sudo apt install tor".cyan());
            println!("   • Tor Browser: Make sure it's running");
            println!();
        }
    }
    
    println!("{} {}", "verify_ssl:".yellow(), config.verify_ssl.to_string().cyan());
    println!();
    
    if config.hyrule_server.contains(".onion") {
        println!("{}", "🧅 Using Tor Hidden Service".green().bold());
        println!("{} All traffic is routed through Tor for maximum privacy", "→".blue());
        println!();
    }
    
    println!("{}", "Available Configuration Keys:".bold());
    println!("  • {}", "server, hyrule_server".dimmed());
    println!("  • {}", "username".dimmed());
    println!("  • {}", "token, auth_token".dimmed());
    println!("  • {}", "private, default_private".dimmed());
    println!("  • {}", "tor, use_tor".dimmed());
    println!("  • {}", "proxy, tor_proxy".dimmed());
    println!("  • {}", "ssl, verify_ssl".dimmed());
    println!();
    
    println!("{}", "Examples:".bold());
    println!("  {}", "triforge config set tor true".cyan());
    println!("  {}", "triforge config set proxy socks5://127.0.0.1:9050".cyan());
    println!("  {}", "triforge config set server http://hyrule4e3tu7pfdkvvca43senvgvgisi6einpe3d3kpidlk3uyjf7lqd.onion".cyan());
    println!();
    
    Ok(())
}
