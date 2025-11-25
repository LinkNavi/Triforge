use colored::*;
use crate::native_git::{Repository, CommitBuilder};

pub fn execute(limit: usize, oneline: bool) -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    
    if !oneline {
        println!("{}", "Commit History".cyan().bold());
        println!("{}", "═".repeat(60).cyan());
        println!();
    }
    
    let mut current = match repo.head_commit() {
        Ok(c) => c,
        Err(_) => {
            println!("{} No commits yet", "→".blue());
            return Ok(());
        }
    };
    
    for i in 0..limit {
        let obj = repo.load_object(&current)?;
        let parsed = CommitBuilder::parse(&obj)?;
        
        if oneline {
            println!("{} {}", 
                current[..8].to_string().yellow(),
                parsed.message.cyan()
            );
        } else {
            println!("{} {}", "commit".yellow().bold(), current.yellow());
            println!("{} {}", "Author:".bold(), parsed.author);
            println!();
            println!("    {}", parsed.message.cyan());
            println!();
        }
        
        // Get parent
        if parsed.parents.is_empty() {
            break;
        }
        current = parsed.parents[0].clone();
    }
    
    Ok(())
}
