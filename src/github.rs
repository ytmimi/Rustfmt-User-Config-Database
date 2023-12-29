use reqwest::header;

/// Details on the endpoint can be found here
/// <https://docs.github.com/en/graphql/guides/forming-calls-with-graphql#the-graphql-endpoint>
const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

const GITHUB_REPOSITORY_QUERY: &str = "
query GitHubRepositoris(
    $query: String!,
    $after: String,
    $limit: Int!,
    $languageOrderBy: LanguageOrder!,
  ) {
    search(
      first: $limit,
      after: $after,
      query: $query,
      type: REPOSITORY
    ) {
      repositoryCount,
      pageInfo {
        hasNextPage,
        endCursor,
        startCursor,
      },
      nodes {
        ... on Repository {
          id,
          nameWithOwner,
          description,
          url,
          archivedAt,
          isFork,
          isLocked,
          pushedAt,
          languages(first: 5, orderBy: $languageOrderBy) {
            totalCount,
            totalSize,
            edges {
              size
              node {
                name
              }
            }
          }
          defaultBranchRef {
            target {
              oid
            }
          }
        }
      }
    }
  }
";

const GITHUB_REPOSITORY_QUERY_VARIABLES: &str = r#"{
    "query": "language:rust topic:rust stars:>=50 template:false archived:false",
    "limit": 10,
    "languageOrderBy": {"field": "SIZE", "direction": "DESC"}
}"#;

pub fn search_github_repositories(api_key: &str, user_agent: &str) -> anyhow::Result<String> {
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
        "operationName": "GitHubRepositoris",
        "query": GITHUB_REPOSITORY_QUERY,
        "variables": GITHUB_REPOSITORY_QUERY_VARIABLES
    });

    println!("Body: {body}");

    let resp = client
        .post(GITHUB_GRAPHQL_URL)
        .body(body.to_string())
        .send()?;

    Ok(resp.text()?)
}
