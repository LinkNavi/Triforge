// src/main.rs
mod api;
mod commands;
mod config;
mod errors;
mod git;
mod native_git;

use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(name = "triforge")]
#[command(version, about = "Standalone distributed version control for Hyrule network", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
    },

    Add {
        paths: Vec<String>,
        #[arg(short, long)]
        all: bool,
    },

    Remove { 
        paths: Vec<String> 
    },

    Commit {
        #[arg(short, long)]
        message: String,
        #[arg(short, long)]
        all: bool,
    },

    Status {
        #[arg(short, long)]
        short: bool,
    },

    Log {
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        oneline: bool,
    },

    Diff {
        from: Option<String>,
        to: Option<String>,
        #[arg(long)]
        name_only: bool,
    },

    Branch {
        #[command(subcommand)]
        action: Option<BranchAction>,
    },

    Checkout {
        target: String,
        #[arg(short = 'b', long)]
        create: bool,
    },

    Merge {
        branch: String,
        #[arg(long)]
        ff_only: bool,
    },

    Push {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(long)]
        private: bool,
    },

    Clone {
        hash: String,
        directory: Option<String>,
        #[arg(short, long)]
        anonymous: bool,
    },

    Pull { 
        remote: Option<String>,
    },

    Info { 
        hash: String 
    },

    List {
        #[arg(long)]
        starred: bool,
        #[arg(long)]
        pinned: bool,
    },

    Verify {
        #[arg(short, long)]
        fix: bool,
    },

    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },

    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    Login,
    Signup,
    Logout,
    Hash,

    Tags {
        #[command(subcommand)]
        action: TagsAction,
    },

    Star { 
        hash: String 
    },

    Unstar { 
        hash: String 
    },

    Pin { 
        hash: String 
    },

    Unpin { 
        hash: String 
    },

    Fork {
        hash: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },

    Search {
        query: String,
        #[arg(short, long)]
        tags: Vec<String>,
        #[arg(long)]
        user: Option<String>,
    },

    Trending {
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    Popular {
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    Stats,
    Nodes,

    Delete {
        hash: String,
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum BranchAction {
    List,
    Create { name: String },
    Delete {
        name: String,
        #[arg(short, long)]
        force: bool,
    },
    Rename { new_name: String },
}

#[derive(Subcommand)]
enum RemoteAction {
    Add { name: String, hash: String },
    Remove { name: String },
    List,
}

#[derive(Subcommand)]
enum ConfigAction {
    Set { key: String, value: String },
    Get { key: String },
    Show,
}

#[derive(Subcommand)]
enum TagsAction {
    Add { hash: String, tags: Vec<String> },
    List { hash: Option<String> },
    Search { tag: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Init { name, description } => {
            commands::init::execute(name, description)?;
        }
        Commands::Add { paths, all } => {
            commands::add::execute(paths, all)?;
        }
        Commands::Remove { paths } => {
            commands::remove::execute(paths)?;
        }
        Commands::Commit { message, all } => {
            commands::commit::execute(&message, all)?;
        }
        Commands::Status { short } => {
            commands::status::execute(short)?;
        }
        Commands::Log { limit, oneline } => {
            commands::log::execute(limit, oneline)?;
        }
        Commands::Diff {
            from,
            to,
            name_only,
        } => {
            commands::diff::execute(from, to, name_only)?;
        }
        Commands::Branch { action } => match action {
            Some(BranchAction::List) | None => commands::branch::list()?,
            Some(BranchAction::Create { name }) => commands::branch::create(&name)?,
            Some(BranchAction::Delete { name, force }) => commands::branch::delete(&name, force)?,
            Some(BranchAction::Rename { new_name }) => commands::branch::rename(&new_name)?,
        },
        Commands::Checkout { target, create } => {
            commands::checkout::execute(&target, create)?;
        }
        Commands::Merge { branch, ff_only } => {
            commands::merge::execute(&branch, ff_only)?;
        }
        Commands::Push {
            name,
            description,
            private,
        } => {
            commands::push::execute(name, description, private, cli.verbose).await?;
        }
        Commands::Clone {
            hash,
            directory,
            anonymous,
        } => {
            commands::clone::execute(&hash, directory, anonymous, cli.verbose).await?;
        }
        Commands::Pull { remote } => {
            commands::pull::execute(remote, cli.verbose).await?;
        }
        Commands::Info { hash } => {
            commands::info::execute(&hash).await?;
        }
        Commands::List { starred, pinned } => {
            commands::list::execute(starred, pinned).await?;
        }
        Commands::Verify { fix } => {
            commands::verify::execute(fix)?;
        }
        Commands::Remote { action } => match action {
            RemoteAction::Add { name, hash } => commands::remote::add(&name, &hash)?,
            RemoteAction::Remove { name } => commands::remote::remove(&name)?,
            RemoteAction::List => commands::remote::list()?,
        },
        Commands::Config { action } => match action {
            ConfigAction::Set { key, value } => commands::config::set(&key, &value)?,
            ConfigAction::Get { key } => commands::config::get(&key)?,
            ConfigAction::Show => commands::config::show()?,
        },
        Commands::Login => {
            commands::auth::login().await?;
        }
        Commands::Signup => {
            commands::auth::signup().await?;
        }
        Commands::Logout => {
            let mut config = config::AppConfig::load()?;
            config.auth_token = None;
            config.username = None;
            config.save()?;
            println!("{} Logged out successfully", "✓".green());
        }
        Commands::Hash => {
            commands::hash::execute()?;
        }
        Commands::Tags { action } => match action {
            TagsAction::Add { hash, tags } => commands::tags::add(&hash, tags).await?,
            TagsAction::List { hash } => commands::tags::list(hash).await?,
            TagsAction::Search { tag } => commands::tags::search(&tag).await?,
        },
        Commands::Star { hash } => {
            commands::star::star(&hash).await?;
        }
        Commands::Unstar { hash } => {
            commands::star::unstar(&hash).await?;
        }
        Commands::Pin { hash } => {
            commands::pin::pin(&hash).await?;
        }
        Commands::Unpin { hash } => {
            commands::pin::unpin(&hash).await?;
        }
        Commands::Fork {
            hash,
            name,
            description,
        } => {
            commands::fork::execute(&hash, name, description).await?;
        }
        Commands::Search { query, tags, user } => {
            commands::search::execute(&query, tags, user).await?;
        }
        Commands::Trending { limit } => {
            commands::trending::execute(limit).await?;
        }
        Commands::Popular { limit } => {
            commands::popular::execute(limit).await?;
        }
        Commands::Stats => {
            commands::stats::execute().await?;
        }
        Commands::Nodes => {
            commands::nodes::execute().await?;
        }
        Commands::Delete { hash, force } => {
            commands::delete::execute(&hash, force).await?;
        }
    }

    Ok(())
}
