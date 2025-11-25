
// TriForge/src/tri/refs.rs
use std::fs;
use std::path::Path;
use anyhow::Result;

/// Set a reference (branch, tag, etc.)
pub fn set_ref(repo_path: &Path, ref_name: &str, object_id: &str) -> Result<()> {
    let ref_path = repo_path.join(ref_name);
    
    if let Some(parent) = ref_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(ref_path, format!("{}\n", object_id))?;
    Ok(())
}

/// Read a reference
pub fn read_ref(repo_path: &Path, ref_name: &str) -> Result<String> {
    let ref_path = repo_path.join(ref_name);
    
    if !ref_path.exists() {
        anyhow::bail!("Reference not found: {}", ref_name);
    }
    
    let content = fs::read_to_string(ref_path)?;
    Ok(content.trim().to_string())
}

/// List all references
pub fn list_refs(repo_path: &Path, prefix: &str) -> Result<Vec<String>> {
    let refs_dir = repo_path.join(prefix);
    let mut refs = Vec::new();
    
    if !refs_dir.exists() {
        return Ok(refs);
    }
    
    fn visit_dir(dir: &Path, base: &Path, refs: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                visit_dir(&path, base, refs)?;
            } else {
                if let Ok(rel_path) = path.strip_prefix(base) {
                    refs.push(rel_path.to_string_lossy().to_string());
                }
            }
        }
        Ok(())
    }
    
    visit_dir(&refs_dir, &repo_path, &mut refs)?;
    Ok(refs)
}
