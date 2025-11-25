
use colored::*;

pub fn execute(from: Option<String>, to: Option<String>, name_only: bool) -> anyhow::Result<()> {
    println!("{}", "Showing changes...".cyan());
    println!();
    println!("{} Diff functionality coming soon", "!".yellow());
    Ok(())
}
