// TriForge/src/commands/private.rs
use colored::*;
use std::fs;
use std::path::Path;
use std::io::{self, Write};
use crate::tri::{TriRepository, crypto, objects, refs, index};

/// Initialize a private .tri repository
pub fn init_private() -> anyhow::Result<()> {
    println!("{}", "Initializing private .tri repository...".cyan().bold());
    println!();
    
    // Check if .tri already exists
    if Path::new(".tri").exists() {
        println!("{} .tri repository already exists", "✓".green());
        return Ok(());
    }
    
    // Initialize .tri repository
    let repo = TriRepository::init(".")?;
    
    println!("{} Created .tri repository structure", "✓".green());
    println!("{} Encryption: {}", "→".blue(), "enabled".green().bold());
    println!("{} Compression: {}", "→".blue(), "enabled".green());
    println!();
    
    // Prompt for encryption password
    print!("{} ", "Set encryption password (leave empty to skip):".yellow());
    io::stdout().flush()?;
    
    let password = rpassword::prompt_password("")?;
    
    if !password.is_empty() {
        // Store password hint (hashed)
        let salt = crypto::EncryptionKey::generate_salt();
        let key = crypto::EncryptionKey::from_password(&password, &salt);
        
        // Store salt in config
        let salt_hex = hex::encode(salt);
        fs::write(repo.path().join("salt"), salt_hex)?;
        
        println!("{} Encryption key set", "✓".green());
        println!("{} Remember your password - it cannot be recovered!", "!".yellow().bold());
    } else {
        println!("{} Skipped encryption setup", "→".blue());
    }
    
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Add files: {}", "triforge add <file>".cyan());
    println!("  2. Commit: {}", "triforge commit -m 'Initial commit'".cyan());
    println!("  3. Push to Hyrule: {}", "triforge push --private".cyan());
    println!();
    
    Ok(())
}

/// Add files to the .tri index
pub fn add_files(paths: Vec<String>) -> anyhow::Result<()> {
    println!("{}", "Adding files to .tri index...".cyan());
    println!();
    
    let repo = TriRepository::open(".")?;
    let mut index = index::Index::load(repo.path())?;
    
    // Get encryption key if repository is encrypted
    let encryption_key = if repo.is_encrypted() {
        Some(get_encryption_key(&repo)?)
    } else {
        None
    };
    
    for path in &paths {
        print!("{} {}... ", "→".blue(), path.yellow());
        io::stdout().flush()?;
        
        // Read file
        let data = fs::read(path)?;
        let metadata = fs::metadata(path)?;
        
        // Store object
        let object_id = objects::store_object(
            repo.path(),
            objects::ObjectType::Blob,
            &data,
            encryption_key.as_ref(),
            repo.config().compression_enabled,
        )?;
        
        // Add to index
        index.add(
            path.clone(),
            object_id.clone(),
            metadata.modified().unwrap()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata.len(),
        );
        
        println!("{} {}", "✓".green(), object_id[..8].to_string().dimmed());
    }
    
    // Save index
    index.save(repo.path())?;
    
    println!();
    println!("{} Added {} files to .tri index", "✓".green(), paths.len().to_string().yellow());
    
    Ok(())
}

/// Commit changes in .tri repository
pub fn commit_private(message: &str) -> anyhow::Result<()> {
    println!("{}", "Creating .tri commit...".cyan());
    println!();
    
    let repo = TriRepository::open(".")?;
    let index = index::Index::load(repo.path())?;
    
    if index.entries.is_empty() {
        anyhow::bail!("Nothing to commit. Use 'triforge add' first.");
    }
    
    // Get encryption key if needed
    let encryption_key = if repo.is_encrypted() {
        Some(get_encryption_key(&repo)?)
    } else {
        None
    };
    
    // Create tree object from index
    let mut tree_entries = Vec::new();
    for entry in index.entries.values() {
        tree_entries.push(format!("blob {} {}", entry.object_id, entry.path));
    }
    
    let tree_data = tree_entries.join("\n");
    let tree_id = objects::store_object(
        repo.path(),
        objects::ObjectType::Tree,
        tree_data.as_bytes(),
        encryption_key.as_ref(),
        repo.config().compression_enabled,
    )?;
    
    println!("{} Created tree: {}", "✓".green(), tree_id[..8].to_string().yellow());
    
    // Get parent commit if exists
    let parent_commit = refs::read_ref(repo.path(), "HEAD")
        .or_else(|_| refs::read_ref(repo.path(), "refs/heads/main"))
        .ok();
    
    // Create commit object
    let mut commit_data = format!("tree {}\n", tree_id);
    
    if let Some(parent) = parent_commit {
        commit_data.push_str(&format!("parent {}\n", parent));
    }
    
    commit_data.push_str(&format!("author TriForge User\n"));
    commit_data.push_str(&format!("committer TriForge User\n"));
    commit_data.push_str(&format!("\n{}\n", message));
    
    let commit_id = objects::store_object(
        repo.path(),
        objects::ObjectType::Commit,
        commit_data.as_bytes(),
        encryption_key.as_ref(),
        repo.config().compression_enabled,
    )?;
    
    // Update HEAD
    refs::set_ref(repo.path(), "refs/heads/main", &commit_id)?;
    refs::set_ref(repo.path(), "HEAD", &commit_id)?;
    
    println!("{} Created commit: {}", "✓".green(), commit_id[..8].to_string().yellow());
    println!();
    println!("{} {}", "Message:".bold(), message.cyan());
    println!("{} {} files", "Files:".bold(), index.entries.len().to_string().yellow());
    
    Ok(())
}

/// Show status of .tri repository
pub fn status_private() -> anyhow::Result<()> {
    println!("{}", ".tri Repository Status".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    
    let repo = TriRepository::open(".")?;
    let index = index::Index::load(repo.path())?;
    
    // Show encryption status
    println!("{} Encryption: {}", "→".blue(), 
        if repo.is_encrypted() { 
            "enabled".green() 
        } else { 
            "disabled".red() 
        }
    );
    println!("{} Compression: {}", "→".blue(), 
        if repo.config().compression_enabled { 
            "enabled".green() 
        } else { 
            "disabled".dimmed() 
        }
    );
    println!();
    
    // Show current branch/commit
    if let Ok(head) = refs::read_ref(repo.path(), "HEAD") {
        println!("{} HEAD: {}", "→".blue(), head[..8].to_string().yellow());
    } else {
        println!("{} HEAD: {}", "→".blue(), "no commits yet".dimmed());
    }
    println!();
    
    // Show staged files
    if index.entries.is_empty() {
        println!("{} No files staged", "→".blue());
    } else {
        println!("{} Staged files:", "→".blue());
        for entry in index.entries.values() {
            println!("  • {} ({})", 
                entry.path.green(), 
                entry.object_id[..8].to_string().dimmed()
            );
        }
    }
    println!();
    
    // Show object count
    let objects = objects::list_objects(repo.path())?;
    println!("{} Total objects: {}", "→".blue(), objects.len().to_string().yellow());
    
    Ok(())
}

/// List commits in .tri repository
pub fn log_private(limit: usize) -> anyhow::Result<()> {
    println!("{}", ".tri Commit History".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    
    let repo = TriRepository::open(".")?;
    
    // Get encryption key if needed
    let encryption_key = if repo.is_encrypted() {
        Some(get_encryption_key(&repo)?)
    } else {
        None
    };
    
    // Get HEAD commit
    let mut current_commit = match refs::read_ref(repo.path(), "HEAD") {
        Ok(id) => id,
        Err(_) => {
            println!("{} No commits yet", "→".blue());
            return Ok(());
        }
    };
    
    let mut count = 0;
    
    while count < limit {
        // Read commit object
        let (obj_type, data) = objects::read_object(
            repo.path(),
            &current_commit,
            encryption_key.as_ref(),
            repo.config().compression_enabled,
        )?;
        
        if !matches!(obj_type, objects::ObjectType::Commit) {
            break;
        }
        
        // Parse commit
        let commit_str = String::from_utf8_lossy(&data);
        let lines: Vec<&str> = commit_str.lines().collect();
        
        // Find message (after blank line)
       
let message = lines
    .iter()
    .skip_while(|l| !l.is_empty())
    .skip(1)
    .cloned()
    .collect::<Vec<_>>()
    .join("\n");

        
        println!("{} {}", "commit".yellow().bold(), current_commit[..8].to_string().yellow());
        
        // Find parent
        let mut has_parent = false;
        for line in &lines {
            if line.starts_with("parent ") {
                current_commit = line[7..].to_string();
                has_parent = true;
                break;
            }
        }
        
        println!("{}", message.trim().cyan());
        println!();
        
        count += 1;
        
        if !has_parent {
            break;
        }
    }
    
    Ok(())
}

/// Helper function to get encryption key from user
fn get_encryption_key(repo: &TriRepository) -> anyhow::Result<crypto::EncryptionKey> {
    // Read salt
    let salt_path = repo.path().join("salt");
    if !salt_path.exists() {
        anyhow::bail!("Repository is encrypted but salt file not found");
    }
    
    let salt_hex = fs::read_to_string(salt_path)?;
    let salt_bytes = hex::decode(salt_hex.trim())?;
    let mut salt = [0u8; 32];
    salt.copy_from_slice(&salt_bytes[..32]);
    
    // Prompt for password
    let password = rpassword::prompt_password("🔐 Encryption password: ")?;
    
    Ok(crypto::EncryptionKey::from_password(&password, &salt))
}

/// Convert .tri repository to regular Git
pub fn tri_to_git() -> anyhow::Result<()> {
    println!("{}", "Converting .tri to .git...".cyan().bold());
    println!();
    
    println!("{} This will decrypt and copy all .tri objects to .git", "→".blue());
    println!("{} Your .tri repository will remain unchanged", "→".blue());
    println!();
    
    print!("{} ", "Continue? [y/N]:".yellow());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if !input.trim().to_lowercase().starts_with('y') {
        println!("Aborted.");
        return Ok(());
    }
    
    let tri_repo = TriRepository::open(".")?;
    let git_repo = crate::git::open_repo()
        .or_else(|_| crate::git::init_repo())?;
    
    // Get encryption key if needed
    let encryption_key = if tri_repo.is_encrypted() {
        Some(get_encryption_key(&tri_repo)?)
    } else {
        None
    };
    
    // Get all .tri objects
    let tri_objects = objects::list_objects(tri_repo.path())?;
    
    println!("{}", "Copying objects...".cyan());
    
    for object_id in &tri_objects {
        print!(".");
        io::stdout().flush()?;
        
        // Read from .tri
        let (obj_type, data) = objects::read_object(
            tri_repo.path(),
            object_id,
            encryption_key.as_ref(),
            tri_repo.config().compression_enabled,
        )?;
        
        // Convert object type
        let git_obj_type = match obj_type {
            objects::ObjectType::Blob => git2::ObjectType::Blob,
            objects::ObjectType::Tree => git2::ObjectType::Tree,
            objects::ObjectType::Commit => git2::ObjectType::Commit,
            objects::ObjectType::Tag => git2::ObjectType::Tag,
        };
        
        // Write to .git
        crate::git::write_object(&git_repo, &data, git_obj_type)?;
    }
    
    println!();
    println!();
    println!("{} Copied {} objects to .git", "✓".green(), tri_objects.len().to_string().yellow());
    
    // Copy HEAD reference
    if let Ok(head) = refs::read_ref(tri_repo.path(), "refs/heads/main") {
        crate::git::set_ref(&git_repo, "refs/heads/main", &head)?;
        println!("{} Updated refs/heads/main", "✓".green());
    }
    
    println!();
    println!("{} Conversion complete!", "✓".green().bold());
    
    Ok(())
}
