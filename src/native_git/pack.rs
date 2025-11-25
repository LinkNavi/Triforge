pub use super::objects::{GitObject, ObjectType};
pub use super::repository::Repository;
pub use super::refs::Refs;
pub use super::tree::TreeBuilder;
pub use super::commit::CommitBuilder;

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

// TriForge/src/native_git/hash.rs
