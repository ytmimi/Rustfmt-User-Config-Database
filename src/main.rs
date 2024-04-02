use anyhow::Context;
use clap::Parser;
use rustfmt_user_config_db::cli::{Cli, Commands};
use rustfmt_user_config_db::{lookup_repositories, store_in_db, GitHubRepoSearch, Repository};
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
            dry_run,
            repo,
            ..
        } => {
            let github_api_token = std::env::var("GITHUB_API_TOKEN")
                .context("Must set GITHUB_API_TOKEN environment variable")?;

            let mut github_search = GitHubRepoSearch::new(&github_api_token);
            github_search
                .repositories_per_page(limit as usize)
                .max_pages(max_pages as usize)
                .min_stars(stars as usize);

            if let Some(name) = repo {
                github_search.repository_name(&name);
            }

            let mut search_results = github_search.search().unwrap();

            // FIXME(ytmim) Need to create the async runtime manually because it doesn't play well with the
            // syncronous `RepoSearchResults`. To get around this it probably makes sense to drop the
            // Iterator impl for `RepoSearchResults` and instead make `get_next_page` async.
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;

            while let Some(repositories) = search_results.get_next_page() {
                if dry_run {
                    for repo in repositories {
                        println!("{repo:#}")
                    }
                    continue;
                }
                runtime.block_on(run_store_in_db(&databse_url, repositories))?;
            }
            println!("Next Token: {:?}", search_results.next_page());
        }
        Commands::ExtractRustfmtToml { limit, dry_run } => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;

            runtime.block_on(extract_rustfmt_confs(&databse_url, limit as i32, dry_run))?;
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

async fn extract_rustfmt_confs(
    connection_str: &str,
    limit: i32,
    dry_run: bool,
) -> anyhow::Result<()> {
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(connection_str)
        .await
        .context("can't connect to database")?;

    let repositories = lookup_repositories(db, limit).await?;

    let temp_dir = tempfile::tempdir()?;
    for repo in repositories {
        if dry_run {
            println!("{repo:#}");
            continue;
        }

        let clone_path = temp_dir.path().join(repo.name_with_owner());
        if let Err(e) = std::fs::create_dir_all(&clone_path) {
            tracing::error!("{e:?} could not create {}", clone_path.display());
            continue;
        }

        let cloned = repo.git_clone(&clone_path)?;
        // TODO(ytmimi) Clone repo etc and store in database
    }
    Ok(())
}
