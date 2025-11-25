use colored::*;
use crate::native_git::Repository;

pub fn execute(short: bool) -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    
    if !short {
        println!("{}", "Repository Status".cyan().bold());
        println!("{}", "═".repeat(60).cyan());
        println!();
    }
    
    // Get current branch/commit
    match repo.head_commit() {
        Ok(commit) => {
            println!("{} HEAD: {}", "→".blue(), commit[..8].to_string().yellow());
        }
        Err(_) => {
            println!("{} HEAD: {}", "→".blue(), "no commits yet".dimmed());
        }
    }
    
    // Show object count
    let objects = repo.list_objects()?;
    println!("{} Objects: {}", "→".blue(), objects.len().to_string().yellow());
    
    // Show size
    let size = repo.size()?;
    println!("{} Size: {} KB", "→".blue(), (size / 1024).to_string().yellow());
    
    Ok(())
}
