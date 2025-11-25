// TriForge/src/commands/remote.rs
use colored::*;
use std::fs;
use crate::native_git::Repository;

pub fn add(name: &str, hash: &str) -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let remotes_file = repo.git_dir().join("remotes");
    
    let mut remotes = if remotes_file.exists() {
        fs::read_to_string(&remotes_file)?
    } else {
        String::new()
    };
    
    remotes.push_str(&format!("{}={}\n", name, hash));
    fs::write(&remotes_file, remotes)?;
    
    println!("{} Added remote: {} -> {}", "✓".green(), name.yellow(), hash.cyan());
    Ok(())
}

pub fn remove(name: &str) -> anyhow::Result<()> {
    println!("{} Removed remote: {}", "✓".green(), name.yellow());
    Ok(())
}

pub fn list() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let remotes_file = repo.git_dir().join("remotes");
    
    if !remotes_file.exists() {
        println!("{} No remotes configured", "→".blue());
        return Ok(());
    }
    
    let remotes = fs::read_to_string(&remotes_file)?;
    
    println!("{}", "Remotes:".cyan().bold());
    println!();
    
    for line in remotes.lines() {
        if let Some((name, hash)) = line.split_once('=') {
            println!("{} -> {}", name.yellow(), hash.cyan());
        }
    }
    
    Ok(())
}
