
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct Refs {
    git_dir: PathBuf,
}

impl Refs {
    pub fn new(git_dir: PathBuf) -> Self {
        Self { git_dir }
    }

    /// Update a reference
    pub fn update(&self, ref_name: &str, hash: &str) -> Result<()> {
        let ref_path = self.git_dir.join(ref_name);
        
        if let Some(parent) = ref_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(ref_path, format!("{}\n", hash))?;
        Ok(())
    }

    /// Read a reference
    pub fn read(&self, ref_name: &str) -> Result<String> {
        let ref_path = self.git_dir.join(ref_name);
        
        if !ref_path.exists() {
            anyhow::bail!("Reference not found: {}", ref_name);
        }
        
        let content = fs::read_to_string(ref_path)?;
        Ok(content.trim().to_string())
    }

    /// Delete a reference
    pub fn delete(&self, ref_name: &str) -> Result<()> {
        let ref_path = self.git_dir.join(ref_name);
        if ref_path.exists() {
            fs::remove_file(ref_path)?;
        }
        Ok(())
    }

    /// List all references with a prefix
    pub fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let prefix_path = self.git_dir.join(prefix);
        let mut refs = Vec::new();
        
        if !prefix_path.exists() {
            return Ok(refs);
        }
        
        self.walk_refs(&prefix_path, prefix, &mut refs)?;
        Ok(refs)
    }

    fn walk_refs(&self, dir: &Path, base: &str, refs: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = entry.file_name();
                let new_base = format!("{}/{}", base, name.to_string_lossy());
                self.walk_refs(&path, &new_base, refs)?;
            } else {
                let name = entry.file_name();
                refs.push(format!("{}/{}", base, name.to_string_lossy()));
            }
        }
        Ok(())
    }

    /// Get HEAD commit hash
    pub fn head(&self) -> Result<String> {
        // Try HEAD file first
        let head_path = self.git_dir.join("HEAD");
        if head_path.exists() {
            let content = fs::read_to_string(head_path)?;
            let content = content.trim();
            
            // If it's a symbolic ref
            if content.starts_with("ref: ") {
                let ref_name = &content[5..];
                return self.read(ref_name);
            }
            
            // Direct hash
            return Ok(content.to_string());
        }
        
        // Try refs/heads/main
        self.read("refs/heads/main")
            .or_else(|_| self.read("refs/heads/master"))
    }
}
