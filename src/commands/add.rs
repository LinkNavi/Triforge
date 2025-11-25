// TriForge/src/commands/add.rs
use colored::*;
use std::fs;
use std::path::Path;
use crate::native_git::Repository;
use walkdir::WalkDir;

pub fn execute(paths: Vec<String>, all: bool) -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    
    let files_to_add = if all {
        collect_all_files(&repo)?
    } else {
        expand_paths(paths)?
    };
    
    if files_to_add.is_empty() {
        println!("{} No files to add", "!".yellow());
        return Ok(());
    }
    
    println!("{}", "Adding files...".cyan());
    
    let mut added_count = 0;
    for path in &files_to_add {
        let path_obj = Path::new(path);
        
        if !path_obj.exists() {
            println!("{} {} (not found)", "!".yellow(), path);
            continue;
        }
        
        if path_obj.is_file() {
            let content = fs::read(path)?;
            let blob = repo.create_blob(&content)?;
            repo.store_object(&blob)?;
            
            println!("{} {}", "+".green(), path.yellow());
            added_count += 1;
        }
    }
    
    println!();
    println!("{} Added {} files", "✓".green(), added_count.to_string().cyan());
    
    Ok(())
}

/// Expand paths to include all files in directories
fn expand_paths(paths: Vec<String>) -> anyhow::Result<Vec<String>> {
    let mut files = Vec::new();
    
    for path in paths {
        let path_obj = Path::new(&path);
        
        if !path_obj.exists() {
            continue;
        }
        
        if path_obj.is_file() {
            files.push(path);
        } else if path_obj.is_dir() {
            // Recursively walk directory
            for entry in WalkDir::new(&path)
                .into_iter()
                .filter_entry(|e| !is_ignored(e.path()))
            {
                let entry = entry?;
                if entry.file_type().is_file() {
                    files.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    
    Ok(files)
}

fn collect_all_files(repo: &Repository) -> anyhow::Result<Vec<String>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(repo.work_dir())
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Ok(rel_path) = entry.path().strip_prefix(repo.work_dir()) {
                files.push(rel_path.to_string_lossy().to_string());
            }
        }
    }
    
    Ok(files)
}

fn is_ignored(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/.git/") || 
    path_str.contains("/.tri/") ||
    path_str.contains("/target/") ||
    path_str.ends_with("/.git") ||
    path_str.ends_with("/.tri")
}
