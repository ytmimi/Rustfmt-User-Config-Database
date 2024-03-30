use anyhow::Context;
use rustfmt_user_config_db::GitHubRepoSearch;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_env("RUSTFMT_LOG"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let github_api_token = std::env::var("GITHUB_API_TOKEN")
        .context("Must set GITHUB_API_TOKEN environment variable")?;

    let mut github_search = GitHubRepoSearch::new(&github_api_token);
    github_search
        .repositories_per_page(10)
        .max_pages(3)
        .min_stars(50);

    let mut search_results = github_search.search().unwrap();

    for repository in &mut search_results {
        println!(
            "GraphQL ID: {}\nName: {}\nLatest Commit: {}\nPushed At: {}\n% Written in Rust {:.2}%\n",
            repository.id(),
            repository.name_with_owner(),
            repository.commit_hash(),
            repository.pushed_at(),
            repository.percent_of_code_in_rust(),
        );
    }
    println!("Next Token: {:?}", search_results.next_page());
    Ok(())
}
