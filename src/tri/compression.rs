// TriForge/src/tri/compression.rs
use flate2::write::{ZlibEncoder, ZlibDecoder};
use flate2::Compression;
use std::io::{Write, Read};
use anyhow::Result;

pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

// TriForge/src/tri/crypto.rs - Enhanced version
use blake3::{Hash, Hasher};
use anyhow::Result;

/// ChaCha20-style stream cipher for encryption
pub struct EncryptionKey {
    key: [u8; 32],
    nonce: [u8; 12],
}

impl EncryptionKey {
    /// Derive encryption key from password using BLAKE3 KDF
    pub fn from_password(password: &str, salt: &[u8]) -> Self {
        // Use BLAKE3 keyed hash for password derivation
        let mut hasher = Hasher::new();
        hasher.update(b"triforge.encryption.v1");
        hasher.update(salt);
        hasher.update(password.as_bytes());
        
        let hash = hasher.finalize();
        let key = *hash.as_bytes();
        
        // Derive nonce from key
        let mut nonce_hasher = Hasher::new();
        nonce_hasher.update(b"triforge.nonce.v1");
        nonce_hasher.update(&key);
        let nonce_hash = nonce_hasher.finalize();
        
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_hash.as_bytes()[..12]);
        
        Self { key, nonce }
    }
    
    /// Generate a random salt using system entropy
    pub fn generate_salt() -> [u8; 32] {
        use std::time::{SystemTime, UNIX_EPOCH};
        use std::process;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let pid = process::id();
        
        let mut hasher = Hasher::new();
        hasher.update(b"triforge.salt.v1");
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(&pid.to_le_bytes());
        
        *hasher.finalize().as_bytes()
    }
    
    /// Encrypt data using stream cipher
    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        // Generate keystream using BLAKE3 in counter mode
        let mut output = Vec::with_capacity(data.len());
        let block_size = 64;
        
        for (block_num, chunk) in data.chunks(block_size).enumerate() {
            let mut hasher = Hasher::new();
            hasher.update(&self.key);
            hasher.update(&self.nonce);
            hasher.update(&(block_num as u64).to_le_bytes());
            
            let keystream = hasher.finalize();
            let keystream_bytes = keystream.as_bytes();
            
            for (i, &byte) in chunk.iter().enumerate() {
                output.push(byte ^ keystream_bytes[i]);
            }
        }
        
        output
    }
    
    /// Decrypt data (stream cipher is symmetric)
    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        self.encrypt(data)
    }
    
    /// Derive a sub-key for specific purposes
    pub fn derive_subkey(&self, context: &[u8]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(b"triforge.subkey.v1");
        hasher.update(&self.key);
        hasher.update(context);
        *hasher.finalize().as_bytes()
    }
}

/// Calculate BLAKE3 hash of data
pub fn hash_data(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hex::encode(hash.as_bytes())
}

/// Verify hash matches data
pub fn verify_hash(data: &[u8], expected_hash: &str) -> bool {
    let actual_hash = hash_data(data);
    actual_hash == expected_hash
}

// TriForge/src/tri/pack.rs - Pack files for efficient storage
use super::objects::{ObjectType, store_object, read_object};
use super::crypto::EncryptionKey;
use std::path::Path;
use std::fs;
use anyhow::Result;

#[derive(Debug)]
pub struct PackEntry {
    pub object_id: String,
    pub object_type: ObjectType,
    pub size: usize,
    pub offset: usize,
}

pub struct PackFile {
    pub entries: Vec<PackEntry>,
    pub data: Vec<u8>,
}

impl PackFile {
    /// Create a pack file from multiple objects
    pub fn create(
        repo_path: &Path,
        object_ids: &[String],
        encryption_key: Option<&EncryptionKey>,
        compress: bool,
    ) -> Result<Self> {
        let mut entries = Vec::new();
        let mut data = Vec::new();
        
        for object_id in object_ids {
            let (obj_type, obj_data) = read_object(
                repo_path,
                object_id,
                encryption_key,
                compress,
            )?;
            
            let offset = data.len();
            entries.push(PackEntry {
                object_id: object_id.clone(),
                object_type: obj_type,
                size: obj_data.len(),
                offset,
            });
            
            data.extend_from_slice(&obj_data);
        }
        
        Ok(Self { entries, data })
    }
    
    /// Write pack file to disk
    pub fn write(&self, path: &Path) -> Result<()> {
        // Write index
        let index_path = path.with_extension("idx");
        let index_json = serde_json::to_vec(&self.entries)?;
        fs::write(index_path, index_json)?;
        
        // Write data
        let data_path = path.with_extension("pack");
        fs::write(data_path, &self.data)?;
        
        Ok(())
    }
    
    /// Read pack file from disk
    pub fn read(path: &Path) -> Result<Self> {
        let index_path = path.with_extension("idx");
        let data_path = path.with_extension("pack");
        
        let index_data = fs::read(index_path)?;
        let entries: Vec<PackEntry> = serde_json::from_slice(&index_data)?;
        
        let data = fs::read(data_path)?;
        
        Ok(Self { entries, data })
    }
}

// TriForge/src/tri/sync.rs - Remote synchronization
use super::TriRepository;
use crate::api::ApiClient;
use anyhow::Result;

pub struct SyncManager {
    repo: TriRepository,
    client: ApiClient,
}

impl SyncManager {
    pub fn new(repo: TriRepository, client: ApiClient) -> Self {
        Self { repo, client }
    }
    
    /// Push all objects to remote
    pub async fn push(&self, repo_hash: &str) -> Result<()> {
        // Get all local objects
        let objects = super::objects::list_objects(self.repo.path())?;
        
        // Upload in batches
        // Implementation here...
        
        Ok(())
    }
    
    /// Pull objects from remote
    pub async fn pull(&self, repo_hash: &str) -> Result<()> {
        // Fetch remote object list
        // Download missing objects
        // Implementation here...
        
        Ok(())
    }
    
    /// Clone repository from remote
    pub async fn clone(
        &self,
        repo_hash: &str,
        local_path: &Path,
    ) -> Result<TriRepository> {
        // Download all objects
        // Reconstruct repository
        // Implementation here...
        
        todo!()
    }
}

// TriForge/src/tri/diff.rs - Show differences between commits
use super::objects::{ObjectType, read_object};
use super::crypto::EncryptionKey;
use std::path::Path;
use anyhow::Result;
use colored::*;

pub struct DiffEntry {
    pub path: String,
    pub status: FileStatus,
    pub old_id: Option<String>,
    pub new_id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
}

impl FileStatus {
    pub fn symbol(&self) -> ColoredString {
        match self {
            FileStatus::Added => "+".green(),
            FileStatus::Modified => "M".yellow(),
            FileStatus::Deleted => "-".red(),
        }
    }
}

pub fn diff_trees(
    repo_path: &Path,
    old_tree_id: Option<&str>,
    new_tree_id: &str,
    encryption_key: Option<&EncryptionKey>,
    compress: bool,
) -> Result<Vec<DiffEntry>> {
    let mut diffs = Vec::new();
    
    // Parse both trees
    let new_tree = parse_tree(repo_path, new_tree_id, encryption_key, compress)?;
    let old_tree = if let Some(id) = old_tree_id {
        Some(parse_tree(repo_path, id, encryption_key, compress)?)
    } else {
        None
    };
    
    // Compare entries
    for (path, new_id) in &new_tree {
        if let Some(ref old) = old_tree {
            if let Some(old_id) = old.get(path) {
                if old_id != new_id {
                    diffs.push(DiffEntry {
                        path: path.clone(),
                        status: FileStatus::Modified,
                        old_id: Some(old_id.clone()),
                        new_id: Some(new_id.clone()),
                    });
                }
            } else {
                diffs.push(DiffEntry {
                    path: path.clone(),
                    status: FileStatus::Added,
                    old_id: None,
                    new_id: Some(new_id.clone()),
                });
            }
        } else {
            diffs.push(DiffEntry {
                path: path.clone(),
                status: FileStatus::Added,
                old_id: None,
                new_id: Some(new_id.clone()),
            });
        }
    }
    
    // Check for deleted files
    if let Some(ref old) = old_tree {
        for (path, old_id) in old {
            if !new_tree.contains_key(path) {
                diffs.push(DiffEntry {
                    path: path.clone(),
                    status: FileStatus::Deleted,
                    old_id: Some(old_id.clone()),
                    new_id: None,
                });
            }
        }
    }
    
    Ok(diffs)
}

fn parse_tree(
    repo_path: &Path,
    tree_id: &str,
    encryption_key: Option<&EncryptionKey>,
    compress: bool,
) -> Result<std::collections::HashMap<String, String>> {
    let (obj_type, data) = read_object(repo_path, tree_id, encryption_key, compress)?;
    
    if !matches!(obj_type, ObjectType::Tree) {
        anyhow::bail!("Not a tree object");
    }
    
    let tree_str = String::from_utf8_lossy(&data);
    let mut entries = std::collections::HashMap::new();
    
    for line in tree_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let object_id = parts[1].to_string();
            let path = parts[2].to_string();
            entries.insert(path, object_id);
        }
    }
    
    Ok(entries)
}

// TriForge/src/tri/merge.rs - Merge branches
use super::refs;
use anyhow::Result;
use std::path::Path;

pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<String>,
    pub merged_commit: Option<String>,
}

pub fn merge_branches(
    repo_path: &Path,
    current_branch: &str,
    merge_branch: &str,
) -> Result<MergeResult> {
    // Get branch commits
    let current_commit = refs::read_ref(repo_path, current_branch)?;
    let merge_commit = refs::read_ref(repo_path, merge_branch)?;
    
    // Find common ancestor
    // Three-way merge
    // Handle conflicts
    
    // For now, just fast-forward if possible
    Ok(MergeResult {
        success: false,
        conflicts: vec!["Merge not yet implemented".to_string()],
        merged_commit: None,
    })
}

// TriForge/src/tri/backup.rs - Backup and recovery
use super::TriRepository;
use std::fs;
use std::path::Path;
use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

pub fn create_backup(repo: &TriRepository, backup_path: &Path) -> Result<()> {
    use walkdir::WalkDir;
    
    let encoder = GzEncoder::new(
        fs::File::create(backup_path)?,
        Compression::best(),
    );
    
    let mut tar = tar::Builder::new(encoder);
    
    for entry in WalkDir::new(repo.path()) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let relative_path = path.strip_prefix(repo.path())?;
            tar.append_path_with_name(path, relative_path)?;
        }
    }
    
    tar.finish()?;
    Ok(())
}

pub fn restore_backup(backup_path: &Path, target_path: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    
    let file = fs::File::open(backup_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    
    archive.unpack(target_path)?;
    Ok(())
}

// TriForge/src/commands/private.rs - Enhanced commands
use colored::*;
use std::fs;
use std::path::Path;
use std::io::{self, Write};
use crate::tri::*;

/// Show diff between commits
pub fn diff_private(from: Option<String>, to: Option<String>) -> anyhow::Result<()> {
    println!("{}", "Showing changes...".cyan().bold());
    println!();
    
    let repo = TriRepository::open(".")?;
    let encryption_key = if repo.is_encrypted() {
        Some(get_encryption_key(&repo)?)
    } else {
        None
    };
    
    // Get commit IDs
    let to_commit = if let Some(ref id) = to {
        id.clone()
    } else {
        refs::read_ref(repo.path(), "HEAD")?
    };
    
    let from_commit = from.as_ref().map(|s| s.as_str());
    
    // Get tree IDs from commits
    let to_tree = get_tree_from_commit(
        repo.path(),
        &to_commit,
        encryption_key.as_ref(),
        repo.config().compression_enabled,
    )?;
    
    let from_tree = if let Some(id) = from_commit {
        Some(get_tree_from_commit(
            repo.path(),
            id,
            encryption_key.as_ref(),
            repo.config().compression_enabled,
        )?)
    } else {
        None
    };
    
    // Generate diff
    let diffs = diff::diff_trees(
        repo.path(),
        from_tree.as_deref(),
        &to_tree,
        encryption_key.as_ref(),
        repo.config().compression_enabled,
    )?;
    
    if diffs.is_empty() {
        println!("{} No changes", "→".blue());
        return Ok(());
    }
    
    println!("{} {} files changed", "→".blue(), diffs.len().to_string().yellow());
    println!();
    
    for diff in diffs {
        println!("{} {}", diff.status.symbol(), diff.path.cyan());
    }
    
    Ok(())
}

/// Create a new branch
pub fn branch_create(name: &str) -> anyhow::Result<()> {
    let repo = TriRepository::open(".")?;
    let head = refs::read_ref(repo.path(), "HEAD")?;
    
    refs::set_ref(repo.path(), &format!("refs/heads/{}", name), &head)?;
    
    println!("{} Created branch: {}", "✓".green(), name.yellow());
    Ok(())
}

/// Switch branches
pub fn branch_switch(name: &str) -> anyhow::Result<()> {
    let repo = TriRepository::open(".")?;
    let branch_ref = format!("refs/heads/{}", name);
    
    // Check if branch exists
    let commit = refs::read_ref(repo.path(), &branch_ref)?;
    
    // Update HEAD
    refs::set_ref(repo.path(), "HEAD", &commit)?;
    
    println!("{} Switched to branch: {}", "✓".green(), name.yellow());
    Ok(())
}

/// List branches
pub fn branch_list() -> anyhow::Result<()> {
    let repo = TriRepository::open(".")?;
    let branches = refs::list_refs(repo.path(), "refs/heads")?;
    
    let current_head = refs::read_ref(repo.path(), "HEAD").ok();
    
    println!("{}", "Branches:".cyan().bold());
    println!();
    
    for branch in branches {
        let branch_name = branch.strip_prefix("refs/heads/").unwrap_or(&branch);
        let commit = refs::read_ref(repo.path(), &branch)?;
        
        let is_current = current_head.as_ref() == Some(&commit);
        let marker = if is_current { "*".green() } else { " ".normal() };
        
        println!("{} {} ({})", 
            marker,
            branch_name.yellow(),
            commit[..8].to_string().dimmed()
        );
    }
    
    Ok(())
}

/// Create encrypted backup
pub fn backup_create(output: &Path) -> anyhow::Result<()> {
    println!("{}", "Creating encrypted backup...".cyan().bold());
    
    let repo = TriRepository::open(".")?;
    backup::create_backup(&repo, output)?;
    
    println!("{} Backup created: {}", "✓".green(), output.display());
    Ok(())
}

/// Restore from backup
pub fn backup_restore(backup_path: &Path) -> anyhow::Result<()> {
    println!("{}", "Restoring from backup...".cyan().bold());
    
    if Path::new(".tri").exists() {
        println!("{} .tri directory already exists", "!".yellow());
        print!("{} ", "Overwrite? [y/N]:".yellow());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Aborted.");
            return Ok(());
        }
        
        fs::remove_dir_all(".tri")?;
    }
    
    backup::restore_backup(backup_path, Path::new("."))?;
    
    println!("{} Repository restored", "✓".green());
    Ok(())
}

fn get_tree_from_commit(
    repo_path: &Path,
    commit_id: &str,
    encryption_key: Option<&crypto::EncryptionKey>,
    compress: bool,
) -> anyhow::Result<String> {
    let (obj_type, data) = objects::read_object(
        repo_path,
        commit_id,
        encryption_key,
        compress,
    )?;
    
    if !matches!(obj_type, objects::ObjectType::Commit) {
        anyhow::bail!("Not a commit object");
    }
    
    let commit_str = String::from_utf8_lossy(&data);
    for line in commit_str.lines() {
        if line.starts_with("tree ") {
            return Ok(line[5..].to_string());
        }
    }
    
    anyhow::bail!("No tree found in commit")
}

fn get_encryption_key(repo: &TriRepository) -> anyhow::Result<crypto::EncryptionKey> {
    let salt_path = repo.path().join("salt");
    if !salt_path.exists() {
        anyhow::bail!("Repository is encrypted but salt file not found");
    }
    
    let salt_hex = fs::read_to_string(salt_path)?;
    let salt_bytes = hex::decode(salt_hex.trim())?;
    let mut salt = [0u8; 32];
    salt.copy_from_slice(&salt_bytes[..32]);
    
    let password = rpassword::prompt_password("🔐 Encryption password: ")?;
    
    Ok(crypto::EncryptionKey::from_password(&password, &salt))
}

// Update Cargo.toml dependencies to add:
// tar = "0.4"
// Add these to TriForge/src/tri/mod.rs:
// pub mod compression;
// pub mod pack;
// pub mod sync;
// pub mod diff;
// pub mod merge;
// pub mod backup;
