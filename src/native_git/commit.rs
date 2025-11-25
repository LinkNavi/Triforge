use super::{GitObject, ObjectType};
use anyhow::Result;
use chrono::{DateTime, Utc};

pub struct CommitBuilder {
    tree: String,
    parents: Vec<String>,
    author: String,
    committer: String,
    message: String,
    timestamp: DateTime<Utc>,
}

impl CommitBuilder {
    pub fn new(tree: String, message: String) -> Self {
        Self {
            tree,
            parents: Vec::new(),
            author: "TriForge User <user@triforge.local>".to_string(),
            committer: "TriForge User <user@triforge.local>".to_string(),
            message,
            timestamp: Utc::now(),
        }
    }

    pub fn parent(mut self, parent: String) -> Self {
        self.parents.push(parent);
        self
    }

    pub fn author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    pub fn committer(mut self, committer: String) -> Self {
        self.committer = committer;
        self
    }

    pub fn build(self) -> Result<GitObject> {
        let mut content = String::new();
        
        content.push_str(&format!("tree {}\n", self.tree));
        
        for parent in &self.parents {
            content.push_str(&format!("parent {}\n", parent));
        }
        
        let timestamp = self.timestamp.timestamp();
        let timezone = "+0000"; // UTC
        
        content.push_str(&format!("author {} {} {}\n", 
            self.author, timestamp, timezone));
        content.push_str(&format!("committer {} {} {}\n", 
            self.committer, timestamp, timezone));
        content.push_str("\n");
        content.push_str(&self.message);
        
        if !self.message.ends_with('\n') {
            content.push('\n');
        }
        
        Ok(GitObject::new(ObjectType::Commit, content.into_bytes()))
    }

    /// Parse a commit object
    pub fn parse(obj: &GitObject) -> Result<ParsedCommit> {
        if obj.obj_type != ObjectType::Commit {
            anyhow::bail!("Not a commit object");
        }

        let content = std::str::from_utf8(&obj.content)?;
        let mut lines = content.lines();
        
        let mut tree = None;
        let mut parents = Vec::new();
        let mut author = None;
        let mut message = String::new();
        let mut in_message = false;

        for line in lines {
            if in_message {
                if !message.is_empty() {
                    message.push('\n');
                }
                message.push_str(line);
            } else if line.is_empty() {
                in_message = true;
            } else if line.starts_with("tree ") {
                tree = Some(line[5..].to_string());
            } else if line.starts_with("parent ") {
                parents.push(line[7..].to_string());
            } else if line.starts_with("author ") {
                author = Some(line[7..].to_string());
            }
        }

        Ok(ParsedCommit {
            tree: tree.ok_or_else(|| anyhow::anyhow!("No tree in commit"))?,
            parents,
            author: author.ok_or_else(|| anyhow::anyhow!("No author in commit"))?,
            message: message.trim().to_string(),
        })
    }
}

#[derive(Debug)]
pub struct ParsedCommit {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub message: String,
}
