
use colored::*;
use crate::native_git::Repository;

pub fn execute(target: &str, create: bool) -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    
    if create {
        let head = repo.head_commit()?;
        repo.refs().update(&format!("refs/heads/{}", target), &head)?;
    }
    
    let branch_ref = format!("refs/heads/{}", target);
    let commit = repo.refs().read(&branch_ref)?;
    
    repo.refs().update("HEAD", &commit)?;
    
    println!("{} Switched to {}", "✓".green(), target.yellow());
    
    Ok(())
}
