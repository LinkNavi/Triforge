use super::{GitObject, ObjectType};
use std::collections::BTreeMap;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: String,
}

pub struct TreeBuilder {
    entries: BTreeMap<String, TreeEntry>,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Add a file (blob) to the tree
    pub fn add_file(&mut self, name: String, hash: String) {
        self.entries.insert(name.clone(), TreeEntry {
            mode: "100644".to_string(), // Regular file
            name,
            hash,
        });
    }

    /// Add a directory (tree) to the tree
    pub fn add_tree(&mut self, name: String, hash: String) {
        self.entries.insert(name.clone(), TreeEntry {
            mode: "040000".to_string(), // Directory
            name,
            hash,
        });
    }

    /// Add an executable file to the tree
    pub fn add_executable(&mut self, name: String, hash: String) {
        self.entries.insert(name.clone(), TreeEntry {
            mode: "100755".to_string(), // Executable
            name,
            hash,
        });
    }

    /// Build the tree object
    pub fn build(self) -> Result<GitObject> {
        let mut content = Vec::new();
        
        for entry in self.entries.values() {
            // Format: mode name\0hash_bytes
            content.extend_from_slice(entry.mode.as_bytes());
            content.push(b' ');
            content.extend_from_slice(entry.name.as_bytes());
            content.push(0);
            
            // Hash as raw bytes (20 bytes for SHA-1)
            let hash_bytes = hex::decode(&entry.hash)?;
            content.extend_from_slice(&hash_bytes);
        }
        
        Ok(GitObject::new(ObjectType::Tree, content))
    }

    /// Parse a tree object
    pub fn parse(obj: &GitObject) -> Result<Vec<TreeEntry>> {
        if obj.obj_type != ObjectType::Tree {
            anyhow::bail!("Not a tree object");
        }

        let mut entries = Vec::new();
        let mut pos = 0;
        let data = &obj.content;

        while pos < data.len() {
            // Read mode
            let space_pos = data[pos..].iter()
                .position(|&b| b == b' ')
                .ok_or_else(|| anyhow::anyhow!("Invalid tree format"))?;
            let mode = std::str::from_utf8(&data[pos..pos + space_pos])?;
            pos += space_pos + 1;

            // Read name
            let null_pos = data[pos..].iter()
                .position(|&b| b == 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid tree format"))?;
            let name = std::str::from_utf8(&data[pos..pos + null_pos])?;
            pos += null_pos + 1;

            // Read hash (20 bytes)
            if pos + 20 > data.len() {
                anyhow::bail!("Invalid tree format");
            }
            let hash = hex::encode(&data[pos..pos + 20]);
            pos += 20;

            entries.push(TreeEntry {
                mode: mode.to_string(),
                name: name.to_string(),
                hash,
            });
        }

        Ok(entries)
    }
}
