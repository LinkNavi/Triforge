// TriForge/src/commands/clone.rs
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use crate::{api, config::AppConfig, git};
use base64::{Engine as _, engine::general_purpose};

pub async fn execute(
    hash: &str,
    directory: Option<String>,
    anonymous: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    println!("{}", "Cloning from Hyrule network...".cyan().bold());
    println!();
    
    // Clean up hash
    let repo_hash = hash
        .trim_start_matches("hyrule://")
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .to_string();
    
    if anonymous {
        println!("{} Using anonymous mode (via Tor)", "→".blue());
        println!("{} This feature is not yet implemented", "!".yellow());
        println!();
    }
    
    println!("{} Repository hash: {}", "→".blue(), repo_hash.yellow());
    
    // Load config
    let config = AppConfig::load()?;
    let client = api::ApiClient::new(config);
    
    // Get repository metadata
    println!("{}", "Fetching repository metadata...".cyan());
    let metadata = client.get_repo(&repo_hash).await?;
    
    println!("{} Name: {}", "✓".green(), metadata.name.yellow());
    if let Some(desc) = &metadata.description {
        println!("{} Description: {}", "  ".blue(), desc);
    }
    println!("{} Size: {} KB", "  ".blue(), (metadata.size / 1024).to_string().yellow());
    println!("{} Replicas: {}", "  ".blue(), metadata.replica_count.to_string().yellow());
    println!("{} Health: {}", "  ".blue(), metadata.health_status.green());
    println!();
    
    // Determine clone directory
    let clone_dir = if let Some(dir) = directory {
        PathBuf::from(dir)
    } else {
        PathBuf::from(&metadata.name)
    };
    
    println!("{} Cloning into: {}", "→".blue(), clone_dir.display().to_string().yellow());
    
    // Create repository
    println!("{}", "Creating local repository...".cyan());
    let repo = git::clone_to_path(&clone_dir)?;
    println!("{} Repository created", "✓".green());
    
    // Get list of objects to download
    println!();
    println!("{}", "Fetching object list...".cyan());
    
    // Get the list of all objects from the server
    let objects_response = client.list_objects(&repo_hash).await?;
    let object_ids = objects_response.objects;
    
    if object_ids.is_empty() {
        println!("{} Repository is empty", "!".yellow());
        return Ok(());
    }
    
    println!("{} Found {} objects", "✓".green(), object_ids.len().to_string().yellow());
    
    // Get HEAD commit
    let head_commit = match client.get_ref(&repo_hash, "refs/heads/main").await {
        Ok(commit) => {
            println!("{} HEAD commit: {}", "✓".green(), commit[..8].to_string().yellow());
            commit.trim().to_string()
        }
        Err(e) => {
            println!("{} No HEAD found: {}", "!".yellow(), e);
            return Ok(());
        }
    };
    
    // Download all objects
    println!();
    println!("{}", "Downloading Git objects...".cyan());
    
    let pb = ProgressBar::new(object_ids.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░")
    );
    
    let mut downloaded = 0;
    let mut failed = 0;
    
    for object_id in &object_ids {
        if verbose {
            pb.set_message(format!("Downloading {}", &object_id[..8]));
        }
        
        match client.download_object(&repo_hash, object_id).await {
            Ok(data) => {
                // Decode base64 if needed
                let object_data = if let Ok(decoded) = general_purpose::STANDARD.decode(&data) {
                    decoded
                } else {
                    data
                };
                
                // Write object to local repository
                match git::write_object(&repo, object_id, &object_data) {
                    Ok(_) => {
                        downloaded += 1;
                        pb.inc(1);
                    }
                    Err(e) => {
                        if verbose {
                            eprintln!("{} Failed to write {}: {}", "✗".red(), object_id, e);
                        }
                        failed += 1;
                        pb.inc(1);
                    }
                }
            }
            Err(e) => {
                if verbose {
                    eprintln!("{} Failed to download {}: {}", "✗".red(), object_id, e);
                }
                failed += 1;
                pb.inc(1);
            }
        }
    }
    
    pb.finish_with_message("Complete!");
    
    println!();
    if failed > 0 {
        println!("{} Failed to download {}/{} objects", 
            "!".yellow(), 
            failed.to_string().red(),
            object_ids.len().to_string().yellow()
        );
    }
    
    // Set HEAD ref
    println!("{}", "Setting HEAD reference...".cyan());
    match git::set_ref(&repo, "refs/heads/main", &head_commit) {
        Ok(_) => println!("{} Set refs/heads/main to {}", "✓".green(), &head_commit[..8].to_string().yellow()),
        Err(e) => {
            println!("{} Failed to set HEAD: {}", "!".yellow(), e);
        }
    }
    
    // Checkout the working directory
    println!("{}", "Checking out files...".cyan());
    match git::checkout_head(&repo) {
        Ok(_) => println!("{} Checked out working directory", "✓".green()),
        Err(e) => {
            println!("{} Failed to checkout: {}", "!".yellow(), e);
            println!("{} You may need to run 'git checkout main' manually", "→".blue());
        }
    }
    
    println!();
    println!("{}", "═".repeat(60).green());
    println!("{}", "✓ Successfully cloned repository!".green().bold());
    println!("{}", "═".repeat(60).green());
    println!();
    println!("{} Downloaded {} objects", "→".blue(), downloaded.to_string().cyan());
    if failed > 0 {
        println!("{} Failed {} objects", "→".blue(), failed.to_string().red());
    }
    println!();
    println!("{} Next steps:", "→".blue());
    println!("  {}", format!("cd {}", clone_dir.display()).cyan());
    println!("  {}", "ls -la       # View files".dimmed());
    println!();
    
    Ok(())
}
