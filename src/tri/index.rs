
// TriForge/src/tri/index.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;

/// File entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub path: String,
    pub object_id: String,
    pub modified_time: u64,
    pub file_size: u64,
}

/// The staging area index
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Index {
    pub entries: HashMap<String, IndexEntry>,
}

impl Index {
    /// Load index from disk
    pub fn load(repo_path: &Path) -> Result<Self> {
        let index_path = repo_path.join("index");
        
        if !index_path.exists() {
            return Ok(Self::default());
        }
        
        let data = fs::read(&index_path)?;
        let index: Index = serde_json::from_slice(&data)?;
        Ok(index)
    }
    
    /// Save index to disk
    pub fn save(&self, repo_path: &Path) -> Result<()> {
        let index_path = repo_path.join("index");
        let data = serde_json::to_vec_pretty(self)?;
        fs::write(&index_path, data)?;
        Ok(())
    }
    
    /// Add a file to the index
    pub fn add(&mut self, path: String, object_id: String, modified_time: u64, file_size: u64) {
        self.entries.insert(path.clone(), IndexEntry {
            path,
            object_id,
            modified_time,
            file_size,
        });
    }
    
    /// Remove a file from the index
    pub fn remove(&mut self, path: &str) -> Option<IndexEntry> {
        self.entries.remove(path)
    }
    
    /// Get an entry from the index
    pub fn get(&self, path: &str) -> Option<&IndexEntry> {
        self.entries.get(path)
    }
    
    /// Clear the index
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
