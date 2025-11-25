// TriForge/src/native_git/repository.rs
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use walkdir::WalkDir;

use super::objects::{GitObject, ObjectType};
use super::refs::Refs;
use super::tree::TreeBuilder;
use super::commit::CommitBuilder;

pub struct Repository {
    work_dir: PathBuf,
    git_dir: PathBuf,
    refs: Refs,
}

impl Repository {
    /// Initialize a new repository
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let work_dir = path.as_ref().to_path_buf();
        let git_dir = work_dir.join(".git");
        
        if git_dir.exists() {
            anyhow::bail!("Git repository already exists");
        }
        
        // Create directory structure
        fs::create_dir_all(&git_dir)?;
        fs::create_dir_all(git_dir.join("objects"))?;
        fs::create_dir_all(git_dir.join("refs/heads"))?;
        fs::create_dir_all(git_dir.join("refs/tags"))?;
        
        // Create HEAD
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;
        
        // Create config
        let config = r#"[core]
	repositoryformatversion = 0
	filemode = true
	bare = false
"#;
        fs::write(git_dir.join("config"), config)?;
        
        let refs = Refs::new(git_dir.clone());
        
        Ok(Self {
            work_dir,
            git_dir,
            refs,
        })
    }

    /// Open an existing repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let work_dir = path.as_ref().to_path_buf();
        let git_dir = work_dir.join(".git");
        
        if !git_dir.exists() {
            anyhow::bail!("Not a git repository");
        }
        
        let refs = Refs::new(git_dir.clone());
        
        Ok(Self {
            work_dir,
            git_dir,
            refs,
        })
    }

    pub fn git_dir(&self) -> &Path {
        &self.git_dir
    }

    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }

    pub fn refs(&self) -> &Refs {
        &self.refs
    }

    /// Store an object
    pub fn store_object(&self, obj: &GitObject) -> Result<String> {
        obj.store(&self.git_dir)?;
        Ok(obj.hash.clone())
    }

    /// Load an object
    pub fn load_object(&self, hash: &str) -> Result<GitObject> {
        GitObject::load(&self.git_dir, hash)
    }

    /// Create a blob from file content
    pub fn create_blob(&self, content: &[u8]) -> Result<GitObject> {
        Ok(GitObject::new(ObjectType::Blob, content.to_vec()))
    }

    /// Get all objects
    pub fn list_objects(&self) -> Result<Vec<String>> {
        GitObject::list_all(&self.git_dir)
    }

    /// Get repository size
    pub fn size(&self) -> Result<u64> {
        let mut total = 0u64;
        for entry in WalkDir::new(&self.git_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                total += entry.metadata()?.len();
            }
        }
        Ok(total)
    }

    /// Get HEAD commit
    pub fn head_commit(&self) -> Result<String> {
        self.refs.head()
    }
}
