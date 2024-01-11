use anyhow::Context;
use rustfmt_user_config_db::search_github_repositories;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let github_api_token = std::env::var("GITHUB_API_TOKEN")
        .context("Must set GITHUB_API_TOKEN environment variable")?;

    let user_agent = std::env::var("GITHUB_USER_AGENT")
        .context("Must set GITHUB_USER_AGENT environment variable")?;

    let graphql_response = search_github_repositories(&github_api_token, &user_agent)?;

    if let Some(errors) = graphql_response.error {
        println!("{errors}")
    }

    let Some(data) = graphql_response.data else {
        return Ok(());
    };

    for repository in data.repositories() {
        println!(
            "GraphQL ID: {}\nName: {}\nLatest Commit: {}\nPushed At: {}\n% Written in Rust {:.2}%\n",
            repository.id(),
            repository.name_with_owner(),
            repository.commit_hash(),
            repository.pushed_at(),
            repository.percent_of_code_in_rust(),
        );
    }
    Ok(())
}
