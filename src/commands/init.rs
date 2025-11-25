use colored::*;
use std::fs;
use crate::native_git;

pub fn execute(name: Option<String>, description: Option<String>) -> anyhow::Result<()> {
    println!("{}", "Initializing TriForge repository...".cyan().bold());
    println!();
    
    let repo = native_git::init(".")?;
    
    println!("{} Initialized empty repository in {}", 
        "✓".green(), 
        repo.work_dir().display().to_string().yellow()
    );
    
    // Store metadata if provided
    if let Some(n) = name {
        fs::write(repo.git_dir().join("name"), n)?;
    }
    if let Some(d) = description {
        fs::write(repo.git_dir().join("description"), d)?;
    }
    
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Add files: {}", "triforge add <files>".cyan());
    println!("  2. Commit changes: {}", "triforge commit -m 'message'".cyan());
    println!("  3. Push to Hyrule: {}", "triforge push".cyan());
    println!();
    
    Ok(())
}
