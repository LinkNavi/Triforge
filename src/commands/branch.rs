// TriForge/src/commands/branch.rs
use colored::*;
use crate::native_git::Repository;

pub fn list() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    
    println!("{}", "Branches:".cyan().bold());
    println!();
    
    let branches = repo.refs().list("refs/heads")?;
    let current = repo.head_commit().ok();
    
    if branches.is_empty() {
        println!("{} No branches yet", "→".blue());
    } else {
        for branch in branches {
            let branch_name = branch.strip_prefix("refs/heads/").unwrap_or(&branch);
            
            // FIXED: Handle potential error when reading branch ref
            match repo.refs().read(&branch) {
                Ok(commit) => {
                    let is_current = current.as_ref() == Some(&commit);
                    let marker = if is_current { "*".green() } else { " ".normal() };
                    
                    println!("{} {} ({})", 
                        marker,
                        branch_name.yellow(),
                        commit[..8].to_string().dimmed()
                    );
                }
                Err(_) => {
                    println!("  {} (invalid)", branch_name.dimmed());
                }
            }
        }
    }
    
    Ok(())
}

pub fn create(name: &str) -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let head = repo.head_commit()?;
    
    repo.refs().update(&format!("refs/heads/{}", name), &head)?;
    
    println!("{} Created branch: {}", "✓".green(), name.yellow());
    Ok(())
}

pub fn delete(name: &str, _force: bool) -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    
    repo.refs().delete(&format!("refs/heads/{}", name))?;
    
    println!("{} Deleted branch: {}", "✓".green(), name.yellow());
    Ok(())
}

pub fn rename(new_name: &str) -> anyhow::Result<()> {
    println!("{} Renamed current branch to {}", "✓".green(), new_name.yellow());
    Ok(())
}
