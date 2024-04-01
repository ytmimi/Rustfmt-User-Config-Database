use anyhow::Context;
use clap::Parser;
use rustfmt_user_config_db::cli::{Cli, Commands};
use rustfmt_user_config_db::{store_in_db, GitHubRepoSearch, Repository};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_env("RUSTFMT_LOG"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let databse_url =
        std::env::var("DATABASE_URL").context("Must set DATABASE_URL environment variable")?;

    let cli = Cli::parse();
    match cli.command {
        Commands::AddRepositories {
            limit,
            max_pages,
            stars,
            ..
        } => {
            let github_api_token = std::env::var("GITHUB_API_TOKEN")
                .context("Must set GITHUB_API_TOKEN environment variable")?;

            let mut github_search = GitHubRepoSearch::new(&github_api_token);
            github_search
                .repositories_per_page(limit as usize)
                .max_pages(max_pages as usize)
                .min_stars(stars as usize);

            let mut search_results = github_search.search().unwrap();

            // FIXME(ytmim) Need to create the async runtime manually because it doesn't play well with the
            // syncronous `RepoSearchResults`. To get around this it probably makes sense to drop the
            // Iterator impl for `RepoSearchResults` and instead make `get_next_page` async.
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            while let Some(repositories) = search_results.get_next_page() {
                runtime.block_on(run_store_in_db(&databse_url, repositories))?;
            }
            println!("Next Token: {:?}", search_results.next_page());
        }
    }

    Ok(())
}

async fn run_store_in_db(
    connection_str: &str,
    repositories: Vec<Repository>,
) -> anyhow::Result<()> {
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(connection_str)
        .await
        .context("can't connect to database")?;

    store_in_db(db, repositories.into_iter()).await
}
