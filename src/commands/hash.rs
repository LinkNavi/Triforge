// TriForge/src/commands/hash.rs
use colored::*;
use crate::native_git::Repository;

pub fn execute() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    
    match repo.head_commit() {
        Ok(commit_hash) => {
            // FIXED: commit_hash is String, use it directly
            println!("{}", commit_hash.green());
        }
        Err(_) => {
            println!("{} No commits yet", "!".yellow());
        }
    }
    
    Ok(())
}
