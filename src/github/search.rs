use super::graphql::{
    github_repository_search_variables, GitHubSearchResult, GraphQLResponse, GITHUB_GRAPHQL_URL,
    GITHUB_REPOSITORY_QUERY,
};
use super::Repository;
use reqwest::header;
use std::collections::VecDeque;
use std::convert::Infallible;
use std::ops::Deref;
use std::str::FromStr;

/// Configure searches for GitHub repositories.
pub struct GitHubRepoSearch<'a> {
    /// API Key used to authenticate your API calls
    api_key: &'a str,
    /// User Agent so GitHub knows which app is making requests
    user_agent: &'a str,
    /// Filter GitHub search results for repositories with this number of stars or higher.
    /// Defaults to 50
    min_stars: usize,
    /// The number of results to return on each page.
    /// Defaults to 100
    limit: usize,
    /// Max number of times to query GitHub for a new page of data.
    /// Defaults to 1.
    max_requests: Option<usize>,
    /// Name of the repository to search for
    repo_name: Option<Repo>,
}

impl<'a> GitHubRepoSearch<'a> {
    pub fn new(api_key: &'a str) -> Self {
        Self {
            api_key,
            user_agent: std::env!("GITHUB_USER_AGENT"),
            min_stars: 50,
            limit: 100,
            max_requests: Some(1),
            repo_name: None,
        }
    }

    /// Set the number of repositories that should be returned on each request to GitHub.
    /// The max value is 1000
    pub fn repositories_per_page(&mut self, limit: usize) -> &mut Self {
        self.limit = std::cmp::min(limit, 1000);
        self
    }

    /// Set the minimum number of stars used to filter repositories
    pub fn min_stars(&mut self, min_stars: usize) -> &mut Self {
        self.min_stars = min_stars;
        self
    }

    /// Set the max number of pages to fetch from GitHub when iterating over [RepoSearchResults].
    pub fn max_pages(&mut self, max_requests: usize) -> &mut Self {
        self.max_requests = Some(max_requests);
        self
    }

    /// Set the repository name to search for
    pub fn repository_name(&mut self, name: &str) -> &mut Self {
        self.repo_name = Some(Repo::from_str(name).expect("infallible conversion"));
        self
    }

    /// Build a [RepoSearchResults] object from your configured [GitHubRepoSearch].
    ///
    /// **Note**: creating a [RepoSearchResults] does not call the GitHub API.
    ///
    /// ```no_run
    /// # use rustfmt_user_config_db::GitHubRepoSearch;
    /// let github_search = GitHubRepoSearch::new(&"MY_API_TOKEN");
    /// for repository in github_search {
    ///     println!(
    ///         "Name: {}\nLatest Commit: {}\nPushed At: {}\n% Written in Rust {:.2}%",
    ///         repository.name_with_owner(),
    ///         repository.commit_hash(),
    ///         repository.pushed_at(),
    ///         repository.percent_of_code_in_rust(),
    ///     );
    /// }
    /// ```
    pub fn search(self) -> Option<RepoSearchResults> {
        let mut headers = header::HeaderMap::new();

        let bearer_token =
            header::HeaderValue::from_str(&format!("Bearer {}", self.api_key)).ok()?;
        headers.insert(header::AUTHORIZATION, bearer_token);
        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent(self.user_agent)
            .default_headers(headers)
            .build()
            .ok()?;

        Some(RepoSearchResults {
            client,
            next_page: None,
            min_stars: self.min_stars,
            limit: self.limit,
            successful_requests_made: 0,
            max_requests: self.max_requests,
            repo_name: self.repo_name,
            buffered_repos: VecDeque::with_capacity(self.limit),
        })
    }
}

pub(super) enum Repo {
    Name(String),
    NameWithOwner(String),
}

impl FromStr for Repo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("/") {
            Ok(Repo::NameWithOwner(s.to_string()))
        } else {
            Ok(Repo::Name(s.to_string()))
        }
    }
}

impl Deref for Repo {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Repo::Name(s) => s.deref(),
            Repo::NameWithOwner(s) => s.deref(),
        }
    }
}

impl<'a> IntoIterator for GitHubRepoSearch<'a> {
    type Item = Repository;
    type IntoIter = RepoSearchResults;

    fn into_iter(self) -> Self::IntoIter {
        self.search().expect("A valid Bearer token was set")
    }
}

/// Used to search for GitHub repositories.
///
/// You can instantiate [RepoSearchResults] by using [GitHubRepoSearch::search]
pub struct RepoSearchResults {
    client: reqwest::blocking::Client,
    next_page: Option<String>,
    min_stars: usize,
    limit: usize,
    successful_requests_made: usize,
    max_requests: Option<usize>,
    repo_name: Option<Repo>,
    buffered_repos: VecDeque<Repository>,
}

impl RepoSearchResults {
    /// Returns the token you can use to query the next page of data if there is one.
    pub fn next_page(&self) -> Option<&str> {
        self.next_page.as_deref()
    }

    /// Makes an API call for the next page of data.
    ///
    /// Each page will contain up to *`n`* repositories, where *`n`* is configured using
    /// [repositories_per_page](GitHubRepoSearch::repositories_per_page).
    ///
    /// [get_next_page](RepoSearchResults::get_next_page) will stop returning results once the
    /// max_requests pages have been returned. The number of pages one is allowed to request
    /// can be configured using [max_pages](GitHubRepoSearch::max_pages)
    pub fn get_next_page(&mut self) -> Option<Vec<Repository>> {
        if let Some(max_requests) = self.max_requests {
            if max_requests <= self.successful_requests_made {
                return None;
            }
        }

        let variables = github_repository_search_variables(
            self.limit,
            self.next_page(),
            Some(self.min_stars),
            self.repo_name.as_ref(),
        );

        let body = serde_json::json!({
            "operationName": "GitHubRepositorySearch",
            "query": GITHUB_REPOSITORY_QUERY,
            "variables": variables
        });

        let request_body = body.to_string();
        tracing::trace!(request_body=?request_body);

        let text = self
            .client
            .post(GITHUB_GRAPHQL_URL)
            .body(request_body)
            .send()
            .and_then(|resp| resp.text())
            .map_err(|err| {
                tracing::error!(request_error=?err);
                err
            })
            .ok()?;

        tracing::trace!(response_body = text);
        let search_results = GraphQLResponse::<GitHubSearchResult>::new(text)
            .map_err(|err| {
                tracing::error!(serialization_error=?err);
                err
            })
            .ok()
            .and_then(|graphql_response| {
                if graphql_response.data.is_some() {
                    graphql_response.data
                } else {
                    tracing::error!(graphql_response_err=?graphql_response.error);
                    None
                }
            })?;

        let total_repository_count = search_results.total_repository_count();
        tracing::debug!(total_repository_count);

        if let Some(next_page) = search_results.next_page() {
            self.next_page = Some(next_page.to_string());
        }

        self.successful_requests_made += 1;
        Some(search_results.into_repositories())
    }
}

impl Iterator for RepoSearchResults {
    type Item = Repository;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(repo_info) = self.buffered_repos.pop_front() {
            return Some(repo_info);
        }

        let repositories = self.get_next_page()?;
        self.buffered_repos.extend(repositories);
        self.buffered_repos.pop_front()
    }
}
