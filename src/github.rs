mod graphql;
use graphql::{github_repository_search_variables, GITHUB_GRAPHQL_URL, GITHUB_REPOSITORY_QUERY};
pub use graphql::{GitHubSearchResult, GraphQLResponse, RepositoryInfo};

use std::collections::VecDeque;

use reqwest::header;

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
    /// Defaults to 10
    limit: usize,
    /// Max number of times to query GitHub for a new page of data.
    /// Defaults to 5.
    max_requests: Option<usize>,
}

impl<'a> GitHubRepoSearch<'a> {
    pub fn new(api_key: &'a str) -> Self {
        Self {
            api_key,
            user_agent: std::env!("GITHUB_USER_AGENT"),
            min_stars: 50,
            limit: 100,
            max_requests: Some(1),
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

    /// Build a [RepoSearchResults] object from your configured [GitHubRepoSearch].
    ///
    /// **Note**: creating a [RepoSearchResults] does not call the GitHub API.
    ///
    /// ```no_run
    /// # use crate::GitHubRepoSearch;
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
            buffered_repos: VecDeque::with_capacity(self.limit),
        })
    }
}

impl<'a> IntoIterator for GitHubRepoSearch<'a> {
    type Item = RepositoryInfo;
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
    buffered_repos: VecDeque<RepositoryInfo>,
}

impl RepoSearchResults {
    /// Returns the token you can use to query the next page of data if there is one.
    pub fn next_page(&self) -> Option<&str> {
        self.next_page.as_deref()
    }

    /// Makes an API call for the next page of data.
    ///
    /// Each page will contain up to `n` repositories, where `n` is configured using
    /// [GitHubRepoSearch::limit].
    ///
    /// [get_next_page](RepoSearchResults::get_next_page) will stop returning results once the
    /// max_requests` pages have been returned. The number of pages one is allowed to request
    /// can be configured using [GitHubRepoSearch::max_requests]
    pub fn get_next_page(&mut self) -> Option<Vec<RepositoryInfo>> {
        if let Some(max_requests) = self.max_requests {
            if max_requests <= self.successful_requests_made {
                return None;
            }
        }

        let variables =
            github_repository_search_variables(self.limit, self.next_page(), Some(self.min_stars));
        let body = serde_json::json!({
            "operationName": "GitHubRepositorySearch",
            "query": GITHUB_REPOSITORY_QUERY,
            "variables": variables
        });

        let request_body = body.to_string();

        let text = self
            .client
            .post(GITHUB_GRAPHQL_URL)
            .body(request_body)
            .send()
            .and_then(|resp| resp.text())
            .map_err(|err| {
                // TODO(ytmimi) log out the error
                return err;
            })
            .ok()?;

        let search_results = GraphQLResponse::<GitHubSearchResult>::new(text)
            .map_err(|err| {
                // TODO(ytmimi) log out the error
                err
            })
            .ok()
            .and_then(|graphql_response| {
                if graphql_response.data.is_some() {
                    graphql_response.data
                } else {
                    // TODO(ytmimi) log out the error
                    None
                }
            })?;

        // TODO(ytmimi) log out the total number of repos that matched the query
        // using search_results.total_repository_count()

        if let Some(next_page) = search_results.next_page() {
            self.next_page = Some(next_page.to_string());
        }

        self.successful_requests_made += 1;
        Some(search_results.into_repositories())
    }
}

impl Iterator for RepoSearchResults {
    type Item = RepositoryInfo;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(repo_info) = self.buffered_repos.pop_front() {
            return Some(repo_info);
        }

        let repositories = self.get_next_page()?;
        self.buffered_repos.extend(repositories);
        self.buffered_repos.pop_front()
    }
}
