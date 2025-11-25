// TriForge/src/git.rs - Wrapper around git2 for push/clone operations
use anyhow::Result;
use git2::{Repository, Oid, ObjectType};
use std::path::Path;
use std::fs;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;
/// Open an existing git repository
pub fn open_repo() -> Result<Repository> {
    Ok(Repository::open(".")?)
}

/// Initialize a new git repository
pub fn init_repo() -> Result<Repository> {
    Ok(Repository::init(".")?)
}

/// Clone to a specific path
pub fn clone_to_path(path: &Path) -> Result<Repository> {
    std::fs::create_dir_all(path)?;
    Ok(Repository::init(path)?)
}

/// Get repository name from current directory or repo
pub fn get_repo_name(repo: &Repository) -> Result<String> {
    // Try to get from remote first
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            if let Some(name) = url.split('/').last() {
                return Ok(name.trim_end_matches(".git").to_string());
            }
        }
    }
    
    // Fall back to directory name
    let workdir = repo.workdir().ok_or_else(|| anyhow::anyhow!("No working directory"))?;
    let name = workdir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?;
    
    Ok(name.to_string())
}

/// Get HEAD commit
pub fn get_head_commit(repo: &Repository) -> Result<git2::Commit> {
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    Ok(commit)
}

/// Get all objects in repository
pub fn get_all_objects(repo: &Repository) -> Result<Vec<Oid>> {
    let mut objects = Vec::new();
    let odb = repo.odb()?;
    
    odb.foreach(|oid| {
        objects.push(*oid);
        true
    })?;
    
    Ok(objects)
}

/// Read object data - FIXED to use correct git2 API
pub fn read_object(repo: &Repository, oid: Oid) -> Result<Vec<u8>> {
    let odb = repo.odb()?;
    let obj = odb.read(oid)?;
    Ok(obj.data().to_vec())
}

/// Get object type as string
pub fn get_object_type(repo: &Repository, oid: Oid) -> Result<String> {
    let obj = repo.find_object(oid, None)?;
    let type_str = match obj.kind() {
        Some(ObjectType::Blob) => "blob",
        Some(ObjectType::Tree) => "tree",
        Some(ObjectType::Commit) => "commit",
        Some(ObjectType::Tag) => "tag",
        _ => "unknown",
    };
    Ok(type_str.to_string())
}

pub fn write_object(repo: &Repository, object_id: &str, data: &[u8]) -> anyhow::Result<()> {
    let repo_path = repo.path();
    let objects_dir = repo_path.join("objects");
    
    // Git stores objects as objects/ab/cdef123...
    let subdir = &object_id[..2];
    let filename = &object_id[2..];
    
    let subdir_path = objects_dir.join(subdir);
    fs::create_dir_all(&subdir_path)?;
    
    let object_path = subdir_path.join(filename);
    
    // Compress and write
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;
    
    fs::write(object_path, compressed)?;
    Ok(())
}

/// Checkout HEAD to working directory  
pub fn checkout_head(repo: &Repository) -> anyhow::Result<()> {
    // First, try to set HEAD to refs/heads/main if it exists
    if let Ok(_) = repo.find_reference("refs/heads/main") {
        repo.set_head("refs/heads/main")?;
    }
    
    let mut checkout_opts = git2::build::CheckoutBuilder::new();
    checkout_opts.force();
    checkout_opts.recreate_missing(true);
    
    repo.checkout_head(Some(&mut checkout_opts))?;
    Ok(())
}

/// Set a reference
pub fn set_ref(repo: &Repository, ref_name: &str, target: &str) -> Result<()> {
    let oid = git2::Oid::from_str(target)?;
    repo.reference(ref_name, oid, true, "triforge update")?;
    Ok(())
}
