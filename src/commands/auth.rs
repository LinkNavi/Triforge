// src/commands/auth.rs
use colored::*;
use std::io::{self, Write};
use crate::{api, config::AppConfig};

pub async fn login() -> anyhow::Result<()> {
    println!("{}", "Login to Hyrule".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    
    let config = AppConfig::load()?;
    check_tor_connectivity(&config)?;
    
    print!("{} ", "Username:".yellow());
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();
    
    let password = rpassword::prompt_password("Password: ")?;
    
    println!();
    println!("{}", "Authenticating...".cyan());
    
    if config.use_tor {
        println!("{} Routing through Tor network...", "🧅".normal());
    }
    
    let client = api::ApiClient::new(config.clone());
    
    match client.login(username, &password).await {
        Ok(response) => {
            let mut new_config = config;
            new_config.auth_token = Some(response.token);
            new_config.username = Some(response.user.username.clone());
            new_config.save()?;
            
            println!("{} Successfully logged in as {}", 
                "✓".green(), 
                response.user.username.yellow().bold()
            );
            println!();
            println!("{} Storage: {} / {} MB", 
                "→".blue(),
                (response.user.storage_used / 1024 / 1024).to_string().cyan(),
                (response.user.storage_quota / 1024 / 1024).to_string().yellow()
            );
            
            if new_config.use_tor {
                println!("{} Connection secured via Tor", "🔒".normal());
            }
            println!();
        }
        Err(e) => {
            println!("{} Login failed: {}", "✗".red(), e);
            
            if config.use_tor && e.to_string().contains("connection") {
                println!();
                println!("{}", "Tor Connection Tips:".yellow().bold());
                println!("{} Make sure Tor is running", "→".blue());
                println!("{} Wait 30-60 seconds for circuit establishment", "→".blue());
                println!("{} Check: {}", "→".blue(), "triforge config show".cyan());
            }
            
            anyhow::bail!("Authentication failed");
        }
    }
    
    Ok(())
}

pub async fn signup() -> anyhow::Result<()> {
    println!("{}", "Sign up for Hyrule".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    
    let config = AppConfig::load()?;
    check_tor_connectivity(&config)?;
    
    print!("{} ", "Username (3-32 chars):".yellow());
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();
    
    if username.len() < 3 || username.len() > 32 {
        anyhow::bail!("Username must be 3-32 characters");
    }
    
    let password = rpassword::prompt_password("Password (min 8 chars): ")?;
    
    if password.len() < 8 {
        anyhow::bail!("Password must be at least 8 characters");
    }
    
    let password_confirm = rpassword::prompt_password("Confirm password: ")?;
    
    if password != password_confirm {
        anyhow::bail!("Passwords do not match");
    }
    
    println!();
    println!("{}", "Creating account...".cyan());
    
    if config.use_tor {
        println!("{} Routing through Tor network...", "🧅".normal());
    }
    
    let client = api::ApiClient::new(config.clone());
    
    match client.signup(username, &password).await {
        Ok(response) => {
            let mut new_config = config;
            new_config.auth_token = Some(response.token);
            new_config.username = Some(response.user.username.clone());
            new_config.save()?;
            
            println!("{} Account created successfully!", "✓".green());
            println!("{} Logged in as {}", 
                "✓".green(), 
                response.user.username.yellow().bold()
            );
            println!();
            println!("{} Email: {}", "→".blue(), response.user.email.dimmed());
            println!("{} You have {} MB of free storage", 
                "→".blue(),
                (response.user.storage_quota / 1024 / 1024).to_string().yellow()
            );
            
            if new_config.use_tor {
                println!("{} Your identity is protected via Tor", "🔒".normal());
            }
            
            println!();
            println!("{}", "Next steps:".bold());
            println!("  1. Initialize a repo: {}", "triforge init".cyan());
            println!("  2. Add files: {}", "triforge add .".cyan());
            println!("  3. Commit: {}", "triforge commit -m 'Initial commit'".cyan());
            println!("  4. Push to Hyrule: {}", "triforge push".cyan());
            println!();
        }
        Err(e) => {
            eprintln!("{} Signup failed: {}", "✗".red(), e);
            eprintln!();
            
            if config.use_tor && e.to_string().contains("connection") {
                eprintln!("{}", "Tor Connection Issues:".yellow().bold());
                eprintln!("  • Make sure Tor is running: {}", "systemctl status tor".cyan());
                eprintln!("  • Wait for circuit: Usually takes 30-60 seconds");
                eprintln!("  • Run health check: {}", "./startup-check.sh".cyan());
            } else {
                eprintln!("{} Troubleshooting:", "→".blue());
                eprintln!("  • Check configuration: {}", "triforge config show".cyan());
                eprintln!("  • Verify server is accessible");
                eprintln!("  • Check network connectivity");
            }
            println!();
            anyhow::bail!("Failed to create account");
        }
    }
    
    Ok(())
}

fn check_tor_connectivity(config: &AppConfig) -> anyhow::Result<()> {
    if config.use_tor {
        println!("{} Tor routing enabled", "🧅".normal());
        
        if !config.check_tor_available() {
            println!();
            println!("{}", "⚠ WARNING: Tor is enabled but not reachable!".yellow().bold());
            println!("{} Tor proxy: {}", "→".blue(), config.tor_proxy.yellow());
            println!();
            println!("{}", "Make sure Tor is running:".bold());
            println!("  • Linux: {}", "sudo systemctl start tor".cyan());
            println!("  • macOS: {}", "brew services start tor".cyan());
            println!("  • Or run: {}", "./startup-check.sh".cyan());
            println!();
            
            print!("{} ", "Continue anyway? [y/N]:".yellow());
            io::stdout().flush()?;
            
            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            
            if !response.trim().to_lowercase().starts_with('y') {
                anyhow::bail!("Aborted - Tor not available");
            }
            
            println!();
        } else {
            println!("{} Tor connection verified", "✓".green());
        }
        
        if config.hyrule_server.contains(".onion") {
            println!("{} Using hidden service: {}", 
                "→".blue(), 
                config.hyrule_server.dimmed()
            );
        }
        
        println!();
    }
    
    Ok(())
}
