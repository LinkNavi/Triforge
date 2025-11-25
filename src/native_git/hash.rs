use sha1::{Sha1, Digest};
use anyhow::Result;

/// Compute SHA-1 hash for Git objects
pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Compute SHA-1 hash with Git object header
pub fn compute_object_hash(obj_type: &str, content: &[u8]) -> String {
    let header = format!("{} {}\0", obj_type, content.len());
    let mut data = header.into_bytes();
    data.extend_from_slice(content);
    compute_hash(&data)
}

/// Verify hash matches data
pub fn verify_hash(data: &[u8], expected: &str) -> bool {
    compute_hash(data) == expected
}


