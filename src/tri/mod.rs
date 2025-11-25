
// TriForge/src/tri/mod.rs
// Private encrypted Git storage system
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

pub mod crypto;
pub mod objects;
pub mod refs;
pub mod index;
pub mod compression;
/// Configuration for .tri repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriConfig {
    pub version: u32,
    pub encryption_enabled: bool,
    pub compression_enabled: bool,
    pub remote_url: Option<String>,
    pub user_id: Option<String>,
}

impl Default for TriConfig {
    fn default() -> Self {
        Self {
            version: 1,
            encryption_enabled: true,
            compression_enabled: true,
            remote_url: None,
            user_id: None,
        }
    }
}

/// Represents a .tri repository
pub struct TriRepository {
    path: PathBuf,
    config: TriConfig,
}

impl TriRepository {
    /// Initialize a new .tri repository
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let tri_path = path.as_ref().join(".tri");
        
        if tri_path.exists() {
            anyhow::bail!(".tri repository already exists");
        }
        
        // Create directory structure
        fs::create_dir_all(&tri_path)?;
        fs::create_dir_all(tri_path.join("objects"))?;
        fs::create_dir_all(tri_path.join("refs").join("heads"))?;
        fs::create_dir_all(tri_path.join("refs").join("tags"))?;
        fs::create_dir_all(tri_path.join("hooks"))?;
        
        // Create config file
        let config = TriConfig::default();
        let config_path = tri_path.join("config.toml");
        let config_str = toml::to_string_pretty(&config)?;
        fs::write(&config_path, config_str)?;
        
        // Create HEAD file
        fs::write(tri_path.join("HEAD"), "ref: refs/heads/main\n")?;
        
        // Create .gitignore for the parent directory
        let gitignore_path = path.as_ref().join(".gitignore");
        let mut gitignore_content = String::new();
        
        if gitignore_path.exists() {
            gitignore_content = fs::read_to_string(&gitignore_path)?;
        }
        
        if !gitignore_content.contains(".tri/") {
            if !gitignore_content.is_empty() && !gitignore_content.ends_with('\n') {
                gitignore_content.push('\n');
            }
            gitignore_content.push_str("# TriForge private repository\n");
            gitignore_content.push_str(".tri/\n");
            fs::write(&gitignore_path, gitignore_content)?;
        }
        
        Ok(Self {
            path: tri_path,
            config,
        })
    }
    
    /// Open an existing .tri repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let tri_path = path.as_ref().join(".tri");
        
        if !tri_path.exists() {
            anyhow::bail!(".tri repository not found. Run 'triforge init --private' first");
        }
        
        let config_path = tri_path.join("config.toml");
        let config_str = fs::read_to_string(&config_path)
            .context("Failed to read .tri config")?;
        let config: TriConfig = toml::from_str(&config_str)?;
        
        Ok(Self {
            path: tri_path,
            config,
        })
    }
    
    /// Get the path to the .tri directory
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    /// Get repository configuration
    pub fn config(&self) -> &TriConfig {
        &self.config
    }
    
    /// Update repository configuration
    pub fn update_config<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut TriConfig),
    {
        f(&mut self.config);
        
        let config_path = self.path.join("config.toml");
        let config_str = toml::to_string_pretty(&self.config)?;
        fs::write(&config_path, config_str)?;
        
        Ok(())
    }
    
    /// Get the objects directory
    pub fn objects_dir(&self) -> PathBuf {
        self.path.join("objects")
    }
    
    /// Get the refs directory
    pub fn refs_dir(&self) -> PathBuf {
        self.path.join("refs")
    }
    
    /// Check if repository is encrypted
    pub fn is_encrypted(&self) -> bool {
        self.config.encryption_enabled
    }
}
