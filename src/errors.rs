// src/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TriforgeError {
    #[error("Not a git repository")]
    NotGitRepo,
    
    #[error("Authentication required. Run 'triforge login' first")]
    NotAuthenticated,
    
    #[error("Repository not found: {0}")]
    RepoNotFound(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Git error: {0}")]
    GitError(String),
    
    #[error("Invalid repository hash: {0}")]
    InvalidHash(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
