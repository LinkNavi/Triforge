
// TriForge/src/tri/crypto.rs
use blake3::{Hash, Hasher};
use anyhow::Result;

/// Encryption key derived from password
pub struct EncryptionKey {
    key: [u8; 32],
}

impl EncryptionKey {
    /// Derive encryption key from password using BLAKE3
    pub fn from_password(password: &str, salt: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        
        let hash = hasher.finalize();
        let key = *hash.as_bytes();
        
        Self { key }
    }
    
    /// Generate a random salt
    pub fn generate_salt() -> [u8; 32] {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let mut hasher = Hasher::new();
        hasher.update(&timestamp.to_le_bytes());
        
        *hasher.finalize().as_bytes()
    }
    
    /// Encrypt data using XOR cipher (for MVP - replace with proper encryption)
    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .zip(self.key.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect()
    }
    
    /// Decrypt data using XOR cipher (for MVP - replace with proper encryption)
    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        // XOR is symmetric
        self.encrypt(data)
    }
}

/// Calculate BLAKE3 hash of data
pub fn hash_data(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hex::encode(hash.as_bytes())
}
