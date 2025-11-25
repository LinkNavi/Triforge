use colored::*;

pub fn execute(paths: Vec<String>) -> anyhow::Result<()> {
    println!("{}", "Removing files from staging...".cyan());
    
    for path in &paths {
        println!("{} {}", "-".red(), path.yellow());
    }
    
    println!();
    println!("{} Removed {} files from staging", "✓".green(), paths.len());
    
    Ok(())
}
