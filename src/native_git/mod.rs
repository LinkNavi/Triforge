// TriForge/src/native_git/mod.rs
pub mod objects;
pub mod refs;
pub mod repository;
pub mod tree;
pub mod commit;
pub mod hash;

// Re-export commonly used types
pub use objects::{GitObject, ObjectType};
pub use repository::Repository;
pub use refs::Refs;
pub use tree::TreeBuilder;
pub use commit::CommitBuilder;

use anyhow::Result;
use std::path::Path;

/// Initialize a new Git repository
pub fn init<P: AsRef<Path>>(path: P) -> Result<Repository> {
    Repository::init(path)
}

/// Open an existing Git repository
pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
    Repository::open(path)
}
