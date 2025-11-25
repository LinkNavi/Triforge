use colored::*;
use std::fs;
use std::collections::BTreeMap;
use crate::native_git::{Repository, TreeBuilder, CommitBuilder};
use walkdir::WalkDir;

pub fn execute(message: &str, all: bool) -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    
    println!("{}", "Creating commit...".cyan());
    println!();
    
    // Build tree from working directory
    let mut tree_builder = TreeBuilder::new();
    let mut file_count = 0;
    
    for entry in WalkDir::new(repo.work_dir())
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            let content = fs::read(entry.path())?;
            let blob = repo.create_blob(&content)?;
            let hash = repo.store_object(&blob)?;
            
            if let Ok(rel_path) = entry.path().strip_prefix(repo.work_dir()) {
                tree_builder.add_file(
                    rel_path.to_string_lossy().to_string(),
                    hash
                );
                file_count += 1;
            }
        }
    }
    
    let tree_obj = tree_builder.build()?;
    let tree_hash = repo.store_object(&tree_obj)?;
    
    // Get parent commit if exists
    let parent = repo.head_commit().ok();
    
    // Build commit
    let mut commit_builder = CommitBuilder::new(tree_hash.clone(), message.to_string());
    
    if let Some(p) = parent {
        commit_builder = commit_builder.parent(p);
    }
    
    let commit_obj = commit_builder.build()?;
    let commit_hash = repo.store_object(&commit_obj)?;
    
    // Update refs
    repo.refs().update("refs/heads/main", &commit_hash)?;
    repo.refs().update("HEAD", &commit_hash)?;
    
    println!("{} Created commit: {}", "✓".green(), commit_hash[..8].to_string().yellow());
    println!();
    println!("{} {}", "Message:".bold(), message.cyan());
    println!("{} {} files", "Files:".bold(), file_count.to_string().yellow());
    
    Ok(())
}

fn is_ignored(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/.git/") || 
    path_str.contains("/.tri/") ||
    path_str.contains("/target/") ||
    path_str.ends_with("/.git") ||
    path_str.ends_with("/.tri")
}
