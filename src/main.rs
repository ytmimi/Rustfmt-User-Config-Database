use anyhow::Context;
use rustfmt_user_config_db::search_github_repositories;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let github_api_token = std::env::var("GITHUB_API_TOKEN")
        .context("Must set GITHUB_API_TOKEN environment variable")?;

    let user_agent = std::env::var("GITHUB_USER_AGENT")
        .context("Must set GITHUB_USER_AGENT environment variable")?;

    let repositores = search_github_repositories(&github_api_token, &user_agent)?;

    println!("{}", repositores);
    Ok(())
}
