use clap::{Parser, Subcommand};

/// Manage the rustfmt user configuration database
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add repositories from GitHub into the database
    #[command(name = "add-repo")]
    AddRepositories {
        /// Limit of how many repositories to fetch on each page
        #[arg(short, long, default_value_t = 100)]
        limit: u16,
        /// Max number of pages to query
        #[arg(short, long, default_value_t = 1)]
        max_pages: u8,
        /// Filter for repositories that have this number of stars or more
        #[arg(short, long, default_value_t = 50)]
        stars: u16,
        /// The name of repository name to add. Either provide a name like `rustfmt`
        /// or provide the repositroy name with the owner like `rust-lang/rustfmt`
        #[arg(short, long)]
        repo: Option<String>,
        /// Print the git repository details instead of storing it in the database
        #[arg(short, long, default_value_t = false)]
        dry_run: bool,
    },
}
