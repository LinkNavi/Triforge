// TriForge/src/tri/objects.rs
use super::crypto::{EncryptionKey, hash_data};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use std::io::{Write, Read};
use flate2::Compression;

/// Object types in .tri repository
#[derive(Debug, Clone, Copy)]
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

/// Store an object in .tri repository
pub fn store_object(
    repo_path: &Path,
    object_type: ObjectType,
    data: &[u8],
    encryption_key: Option<&EncryptionKey>,
    compress: bool,
) -> Result<String> {
    // Create object header: "type size\0data"
    let header = format!("{} {}\0", object_type.as_str(), data.len());
    let mut full_data = header.into_bytes();
    full_data.extend_from_slice(data);
    
    // Calculate hash before encryption
    let object_id = hash_data(&full_data);
    
    // Compress if enabled
    let processed_data = if compress {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&full_data)?;
        encoder.finish()?
    } else {
        full_data
    };
    
    // Encrypt if key provided
    let final_data = if let Some(key) = encryption_key {
        key.encrypt(&processed_data)
    } else {
        processed_data
    };
    
    // Store in objects directory: objects/ab/cdef123...
    let objects_dir = repo_path.join("objects");
    let subdir = &object_id[..2];
    let filename = &object_id[2..];
    
    let subdir_path = objects_dir.join(subdir);
    fs::create_dir_all(&subdir_path)?;
    
    let object_path = subdir_path.join(filename);
    fs::write(&object_path, final_data)?;
    
    Ok(object_id)
}

/// Read an object from .tri repository
pub fn read_object(
    repo_path: &Path,
    object_id: &str,
    encryption_key: Option<&EncryptionKey>,
    compressed: bool,
) -> Result<(ObjectType, Vec<u8>)> {
    let objects_dir = repo_path.join("objects");
    let subdir = &object_id[..2];
    let filename = &object_id[2..];
    
    let object_path = objects_dir.join(subdir).join(filename);
    
    if !object_path.exists() {
        anyhow::bail!("Object not found: {}", object_id);
    }
    
    let mut data = fs::read(&object_path)?;
    
    // Decrypt if key provided
    if let Some(key) = encryption_key {
        data = key.decrypt(&data);
    }
    
    // Decompress if enabled
    let processed_data = if compressed {
        let mut decoder = ZlibDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        decompressed
    } else {
        data
    };
    
    // Parse object header
    let null_pos = processed_data.iter().position(|&b| b == 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid object format"))?;
    
    let header = std::str::from_utf8(&processed_data[..null_pos])?;
    let mut parts = header.split(' ');
    
    let obj_type_str = parts.next()
        .ok_or_else(|| anyhow::anyhow!("Missing object type"))?;
    
    let obj_type = ObjectType::from_str(obj_type_str)
        .ok_or_else(|| anyhow::anyhow!("Invalid object type: {}", obj_type_str))?;
    
    let content = processed_data[null_pos + 1..].to_vec();
    
    Ok((obj_type, content))
}

/// List all objects in repository
pub fn list_objects(repo_path: &Path) -> Result<Vec<String>> {
    let objects_dir = repo_path.join("objects");
    let mut objects = Vec::new();
    
    if !objects_dir.exists() {
        return Ok(objects);
    }
    
    for entry in fs::read_dir(objects_dir)? {
        let entry = entry?;
        let subdir_name = entry.file_name();
        let subdir_path = entry.path();
        
        if subdir_path.is_dir() {
            for obj_entry in fs::read_dir(subdir_path)? {
                let obj_entry = obj_entry?;
                let obj_name = obj_entry.file_name();
                let object_id = format!(
                    "{}{}",
                    subdir_name.to_string_lossy(),
                    obj_name.to_string_lossy()
                );
                objects.push(object_id);
            }
        }
    }
    
    Ok(objects)
}
