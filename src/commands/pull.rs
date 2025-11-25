// TriForge/src/commands/pull.rs - Updated to use correct API
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use crate::{api, config::AppConfig, git};
use base64::{Engine as _, engine::general_purpose};
use anyhow::Result;

pub async fn execute(remote: Option<String>, verbose: bool) -> Result<()> {
    println!("{}", "Pulling from Hyrule network...".cyan().bold());
    println!();
    
    let config = AppConfig::load()?;
    let client = api::ApiClient::new(config);
    let repo = git::open_repo()?;
    
    // Get remote hash from git config or remote name
    let repo_hash = if let Some(r) = remote {
        // Try to read from git config: hyrule.{remote}.hash
        get_remote_hash(&r)?
    } else {
        // Default to 'origin'
        get_remote_hash("origin")?
    };
    
    if verbose {
        println!("{} Repository: {}", "→".blue(), repo_hash.yellow());
    }
    
    // Get repository metadata
    let metadata = client.get_repo(&repo_hash).await?;
    println!("{} Fetching from: {}", "→".blue(), metadata.name.yellow());
    
    // Get remote HEAD
    let remote_head = client.get_ref(&repo_hash, "refs/heads/main").await?;
    println!("{} Remote HEAD: {}", "→".blue(), remote_head[..8].to_string().yellow());
    
    // Get local HEAD
    let local_head = git::get_head_commit(&repo)?;
    let local_head_id = local_head.id().to_string();
    
    if remote_head == local_head_id {
        println!("{} Already up to date", "✓".green());
        return Ok(());
    }
    
    // Get list of objects
    println!("{}", "Fetching object list...".cyan());
    let objects_response = client.list_objects(&repo_hash).await?;
    let remote_objects = objects_response.objects;
    
    println!("{} Remote has {} objects", "→".blue(), remote_objects.len().to_string().yellow());
    
    // Get local objects
    let local_objects = git::get_all_objects(&repo)?;
    let local_oids: std::collections::HashSet<_> = local_objects
        .iter()
        .map(|oid| oid.to_string())
        .collect();
    
    // Find missing objects
    let missing: Vec<_> = remote_objects
        .iter()
        .filter(|oid| !local_oids.contains(*oid))
        .collect();
    
    if missing.is_empty() {
        println!("{} No new objects to fetch", "✓".green());
    } else {
        println!("{} Downloading {} new objects...", "→".blue(), missing.len().to_string().yellow());
        
        let pb = ProgressBar::new(missing.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("█▓░")
        );
        
        for oid in &missing {
            if verbose {
                pb.set_message(format!("Downloading {}", &oid[..8]));
            }
            
            match client.download_object(&repo_hash, oid).await {
                Ok(data) => {
                    let object_data = if let Ok(decoded) = general_purpose::STANDARD.decode(&data) {
                        decoded
                    } else {
                        data
                    };
                    
                    if let Err(e) = git::write_object(&repo, oid, &object_data) {
                        if verbose {
                            eprintln!("{} Failed to write {}: {}", "✗".red(), oid, e);
                        }
                    }
                    pb.inc(1);
                }
                Err(e) => {
                    if verbose {
                        eprintln!("{} Failed to download {}: {}", "✗".red(), oid, e);
                    }
                    pb.inc(1);
                }
            }
        }
        
        pb.finish_with_message("Complete!");
    }
    
    // Update refs
    println!();
    println!("{}", "Updating references...".cyan());
    git::set_ref(&repo, "refs/remotes/origin/main", &remote_head)?;
    println!("{} Updated refs/remotes/origin/main", "✓".green());
    
    // Merge changes
    println!("{}", "Merging changes...".cyan());
    println!("{} Fast-forward merge not yet implemented", "!".yellow());
    println!("{} Run manually: {}", "→".blue(), "git merge origin/main".cyan());
    
    println!();
    println!("{} Pull complete!", "✓".green().bold());
    
    Ok(())
}

fn get_remote_hash(remote_name: &str) -> Result<String> {
    use std::process::Command;
    
    let output = Command::new("git")
        .args(&["config", "--get", &format!("remote.{}.hyrule-hash", remote_name)])
        .output()?;
    
    if output.status.success() {
        let hash = String::from_utf8(output.stdout)?.trim().to_string();
        if !hash.is_empty() {
            return Ok(hash);
        }
    }
    
    anyhow::bail!(
        "Remote '{}' not configured. Set with: git config remote.{}.hyrule-hash <hash>",
        remote_name,
        remote_name
    );
}
