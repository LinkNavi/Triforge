// TriForge/src/commands/push.rs - Fixed with better error handling
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use crate::{api, config::AppConfig, git};
use base64::{Engine as _, engine::general_purpose};

pub async fn execute(
    name: Option<String>,
    description: Option<String>,
    private: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    println!("{}", "Pushing to Hyrule network...".cyan().bold());
    println!();
    
    // Load config
    let config = AppConfig::load()?;
    
    if config.auth_token.is_none() {
        anyhow::bail!("Not authenticated. Run 'triforge login' first.");
    }
    
    println!("{} Using server: {}", "→".blue(), config.hyrule_server.yellow());
    
    // Open repository
    let repo = git::open_repo()?;
    
    // Get repository name
    let repo_name = if let Some(n) = name {
        n
    } else {
        git::get_repo_name(&repo)?
    };
    
    println!("{} Repository: {}", "→".blue(), repo_name.yellow());
    if let Some(desc) = &description {
        println!("{} Description: {}", "→".blue(), desc);
    }
    println!("{} Privacy: {}", "→".blue(), if private { "Private".red() } else { "Public".green() });
    println!();
    
    // Get HEAD commit
    println!("{}", "Analyzing repository...".cyan());
    let head_commit = git::get_head_commit(&repo)?;
    let head_id = head_commit.id().to_string();
    println!("{} HEAD: {}", "✓".green(), head_id[..8].to_string().yellow());
    
    // Get all objects
    println!("{}", "Collecting Git objects...".cyan());
    let objects = git::get_all_objects(&repo)?;
    println!("{} Found {} objects", "✓".green(), objects.len().to_string().yellow());
    
    if objects.is_empty() {
        anyhow::bail!("No objects to push. Make sure you have committed changes.");
    }
    
    // Create repository on Hyrule
    println!("{}", "Creating repository on Hyrule...".cyan());
    let client = api::ApiClient::new(config);
    
    let req = api::CreateRepoRequest {
        name: repo_name.clone(),
        description: description.clone(),
        storage_tier: "free".to_string(),
        is_private: private,
    };
    
    let response = client.create_repo(req).await?;
    
    println!("{} {}", "✓".green(), response.message);
    println!();
    println!("{} Repository hash: {}", "→".blue(), response.repo_hash.green().bold());
    
    // Upload objects in batches
    println!();
    println!("{}", "Uploading Git objects...".cyan());
    
    let pb = ProgressBar::new(objects.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░")
    );
    
    // Batch upload for efficiency
    let batch_size = 1;
    let mut uploaded_count = 0;
    let mut failed_count = 0;
    let total_objects = objects.len();
    
    for chunk in objects.chunks(batch_size) {
        let mut batch = Vec::new();
        
        for oid in chunk {
            if verbose {
                pb.set_message(format!("Processing {}", oid));
            }
            
            match git::read_object(&repo, *oid) {
                Ok(data) => {
                    match git::get_object_type(&repo, *oid) {
                        Ok(object_type) => {
                            // Encode data as base64
                            let encoded_data = general_purpose::STANDARD.encode(&data);
                            
                            batch.push(api::UploadObjectRequest {
                                object_id: oid.to_string(),
                                object_type,
                                data: encoded_data,
                            });
                        }
                        Err(e) => {
                            if verbose {
                                eprintln!("{} Failed to get type for {}: {}", "!".yellow(), oid, e);
                            }
                            failed_count += 1;
                        }
                    }
                }
                Err(e) => {
                    if verbose {
                        eprintln!("{} Failed to read object {}: {}", "!".yellow(), oid, e);
                    }
                    failed_count += 1;
                }
            }
        }
        
        if !batch.is_empty() {
            let batch_len = batch.len();
            
            // Upload batch
            match client.batch_upload_objects(&response.repo_hash, batch).await {
                Ok(result) => {
                    uploaded_count += result.uploaded;
                    pb.inc(result.uploaded as u64);
                    
                    if !result.failed.is_empty() {
                        failed_count += result.failed.len();
                        if verbose {
                            for failed_id in &result.failed {
                                eprintln!("{} Failed to upload: {}", "✗".red(), failed_id);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} Batch upload failed: {}", "✗".red(), e);
                    eprintln!("{} This might be a network or server issue", "→".blue());
                    failed_count += batch_len;
                    pb.inc(batch_len as u64);
                }
            }
        }
    }
    
    pb.finish_with_message("Complete!");
    
    println!();
    
    if failed_count > 0 {
        println!("{} Failed to upload {}/{} objects", 
            "!".yellow(), 
            failed_count.to_string().red(),
            total_objects.to_string().yellow()
        );
    }
    
    if uploaded_count == 0 {
        println!("{}", "═".repeat(60).red());
        println!("{}", "✗ Upload failed - no objects were uploaded!".red().bold());
        println!("{}", "═".repeat(60).red());
        println!();
        println!("{} Possible issues:", "→".blue());
        println!("  • Server might not be running or reachable");
        println!("  • Check server URL: {}", "triforge config show".cyan());
        println!("  • Try setting correct server: {}", "triforge config set server http://localhost:3000".cyan());
        println!();
        anyhow::bail!("Push failed - no objects uploaded");
    }
    
    // Update HEAD ref
    println!("{}", "Updating references...".cyan());
    match client.update_ref(&response.repo_hash, "refs/heads/main", &head_id).await {
        Ok(_) => println!("{} Updated refs/heads/main", "✓".green()),
        Err(e) => {
            eprintln!("{} Failed to update ref: {}", "!".yellow(), e);
            eprintln!("{} Objects were uploaded but HEAD may not be set", "→".blue());
        }
    }
    
    println!();
    println!("{}", "═".repeat(60).green());
    println!("{}", "✓ Successfully pushed to Hyrule network!".green().bold());
    println!("{}", "═".repeat(60).green());
    println!();
    println!("{} Uploaded {} objects", "→".blue(), uploaded_count.to_string().cyan());
    if failed_count > 0 {
        println!("{} Failed {} objects", "→".blue(), failed_count.to_string().red());
    }
    println!();
    println!("{} Clone with:", "→".blue());
    println!("  {}", format!("triforge clone {}", response.repo_hash).cyan());
    println!();
    println!("{} View on web:", "→".blue());
    println!("  {}", format!("http://localhost:3000/r/{}", response.repo_hash).cyan());
    println!();
    
    Ok(())
}
