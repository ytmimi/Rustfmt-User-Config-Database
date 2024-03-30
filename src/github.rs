mod graphql;
use graphql::{github_repository_search_variables, GITHUB_GRAPHQL_URL, GITHUB_REPOSITORY_QUERY};
pub use graphql::{GitHubSearchResult, GraphQLResponse};

use reqwest::header;

pub fn search_github_repositories(
    api_key: &str,
    user_agent: &str,
) -> anyhow::Result<GraphQLResponse<GitHubSearchResult>> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {api_key}"))?,
    );

    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent(user_agent)
        .default_headers(headers)
        .build()?;

    let body = serde_json::json!({
        "operationName": "GitHubRepositorySearch",
        "query": GITHUB_REPOSITORY_QUERY,
        "variables": github_repository_search_variables(10, None, None)
    });

    let resp = client
        .post(GITHUB_GRAPHQL_URL)
        .body(body.to_string())
        .send()?;

    let text = resp.text()?;
    Ok(GraphQLResponse::new(text)?)
}
