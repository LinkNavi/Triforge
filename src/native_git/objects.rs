// TriForge/src/native_git/objects.rs
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use std::io::{Write, Read};

use super::hash;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
            ObjectType::Commit => "commit",
            ObjectType::Tag => "tag",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "blob" => Some(ObjectType::Blob),
            "tree" => Some(ObjectType::Tree),
            "commit" => Some(ObjectType::Commit),
            "tag" => Some(ObjectType::Tag),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GitObject {
    pub obj_type: ObjectType,
    pub content: Vec<u8>,
    pub hash: String,
}

impl GitObject {
    /// Create a new Git object
    pub fn new(obj_type: ObjectType, content: Vec<u8>) -> Self {
        let hash = hash::compute_object_hash(obj_type.as_str(), &content);
        Self {
            obj_type,
            content,
            hash,
        }
    }

    /// Get the full object data with header (for storage)
    pub fn data(&self) -> Vec<u8> {
        let header = format!("{} {}\0", self.obj_type.as_str(), self.content.len());
        let mut data = header.into_bytes();
        data.extend_from_slice(&self.content);
        data
    }

    /// Store this object in a Git repository
    pub fn store(&self, git_dir: &Path) -> Result<()> {
        let objects_dir = git_dir.join("objects");
        let (dir, file) = self.path_components();
        
        let obj_dir = objects_dir.join(dir);
        fs::create_dir_all(&obj_dir)?;
        
        let obj_path = obj_dir.join(file);
        
        // Compress with zlib
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.data())?;
        let compressed = encoder.finish()?;
        
        fs::write(obj_path, compressed)?;
        Ok(())
    }

    /// Load an object from a Git repository
    pub fn load(git_dir: &Path, hash: &str) -> Result<Self> {
        let objects_dir = git_dir.join("objects");
        let obj_path = objects_dir.join(&hash[..2]).join(&hash[2..]);
        
        if !obj_path.exists() {
            anyhow::bail!("Object not found: {}", hash);
        }

        let compressed = fs::read(&obj_path)?;
        
        // Decompress
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut data = Vec::new();
        decoder.read_to_end(&mut data)?;
        
        // Parse header
        let null_pos = data.iter()
            .position(|&b| b == 0)
            .context("Invalid object format")?;
        
        let header = std::str::from_utf8(&data[..null_pos])?;
        let mut parts = header.split(' ');
        
        let obj_type_str = parts.next().context("Missing object type")?;
        let obj_type = ObjectType::from_str(obj_type_str)
            .context("Invalid object type")?;
        
        let content = data[null_pos + 1..].to_vec();
        
        Ok(Self {
            obj_type,
            content,
            hash: hash.to_string(),
        })
    }

    /// Get path components (2-char dir, rest as filename)
    fn path_components(&self) -> (&str, &str) {
        (&self.hash[..2], &self.hash[2..])
    }

    /// List all objects in a repository
    pub fn list_all(git_dir: &Path) -> Result<Vec<String>> {
        let objects_dir = git_dir.join("objects");
        let mut objects = Vec::new();
        
        if !objects_dir.exists() {
            return Ok(objects);
        }
        
        for entry in fs::read_dir(objects_dir)? {
            let entry = entry?;
            let dir_name = entry.file_name();
            let dir_path = entry.path();
            
            if dir_path.is_dir() && dir_name.len() == 2 {
                for obj_entry in fs::read_dir(dir_path)? {
                    let obj_entry = obj_entry?;
                    let obj_name = obj_entry.file_name();
                    let hash = format!("{}{}", 
                        dir_name.to_string_lossy(), 
                        obj_name.to_string_lossy()
                    );
                    objects.push(hash);
                }
            }
        }
        
        Ok(objects)
    }
}
