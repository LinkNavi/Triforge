use colored::*;
use crate::native_git::Repository;

pub fn execute(fix: bool) -> anyhow::Result<()> {
    println!("{}", "Verifying repository...".cyan());
    println!();
    
    let repo = Repository::open(".")?;
    let objects = repo.list_objects()?;
    
    println!("{} Checking {} objects...", "→".blue(), objects.len());
    
    let mut valid = 0;
    let mut invalid = 0;
    
    for obj_hash in &objects {
        match repo.load_object(obj_hash) {
            Ok(_) => valid += 1,
            Err(_) => {
                invalid += 1;
                println!("{} Invalid object: {}", "✗".red(), obj_hash[..8].to_string());
            }
        }
    }
    
    println!();
    println!("{} Valid: {}", "✓".green(), valid.to_string().cyan());
    if invalid > 0 {
        println!("{} Invalid: {}", "✗".red(), invalid.to_string().red());
    }
    
    Ok(())
}
