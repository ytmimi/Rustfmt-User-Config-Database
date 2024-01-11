use serde::de::DeserializeOwned;
use serde::Deserialize;
use time::OffsetDateTime;

/// Details on the endpoint can be found here
/// <https://docs.github.com/en/graphql/guides/forming-calls-with-graphql#the-graphql-endpoint>
pub(crate) const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

pub(crate) const GITHUB_REPOSITORY_QUERY: &str = "
query GitHubRepositorySearch(
  # The search string to look for. GitHub search syntax is supported.
  $gitHubSearchString: String!
  # Returns the first n elements from the list.
  $limit: Int!
  # Returns the elements in the list that come after the specified cursor.
  # Check the `endCursor` on the returned pageInfo
  $cursorOffset: String
  # Ordering options for language connections.
  $languageOrderBy: LanguageOrder!
) {
  search(first: $limit, after: $cursorOffset, query: $gitHubSearchString, type: REPOSITORY) {
    repositoryCount
    pageInfo {
      hasNextPage
      endCursor
    }
    nodes {
      ... on Repository {
        id
        nameWithOwner
        description
        url
        archivedAt
        isFork
        isLocked
        pushedAt
        updatedAt
        languages(first: 5, orderBy: $languageOrderBy) {
          totalCount
          totalSize
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

pub(crate) fn github_repository_search_variables(
    limit: usize,
    cursor_offset: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
      "gitHubSearchString": "language:rust topic:rust stars:>=50 template:false archived:false",
      "limit": limit,
      "cursorOffset": cursor_offset,
      "languageOrderBy": {"field": "SIZE", "direction": "DESC"}
    })
}

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub error: Option<serde_json::Value>,
}

impl<T> GraphQLResponse<T>
where
    T: DeserializeOwned,
{
    pub fn new(source: String) -> Result<Self, serde_json::Error> {
        serde_json::from_str(&source)
    }
}

/// Information about pagination in a connection.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    /// When paginating forwards, are there more items?
    #[allow(unused)]
    // FIXME(ytmimi): remove #[allow(unused)] after I add pagination support.
    has_next_page: bool,
    /// When paginating forwards, the cursor to continue
    #[allow(unused)]
    // FIXME(ytmimi): remove #[allow(unused)] after I add pagination support.
    end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubSearchResult {
    search: GitHubSearchResultInner,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubSearchResultInner {
    /// The total number of repositories that matched the search query
    repository_count: usize,
    /// Information to aid in pagination.
    #[allow(unused)]
    // FIXME(ytmimi): remove #[allow(unused)] after I add pagination support.
    page_info: PageInfo,
    #[serde(rename = "nodes")]
    /// A list of repositories
    repositories: Vec<RepositoryInfo>,
}

impl GitHubSearchResult {
    /// The total number of repositories that met the search critera.
    pub fn total_repository_count(&self) -> usize {
        self.search.repository_count
    }

    /// All the repositories returned in this page of data.
    pub fn repositories(&self) -> &[RepositoryInfo] {
        &self.search.repositories
    }

    /// Token for the next page of data if it exists.
    #[allow(unused)]
    // FIXME(ytmimi): remove #[allow(unused)] after I add pagination support.
    fn next_page(&self) -> Option<&str> {
        self.search.page_info.end_cursor.as_deref()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInfo {
    /// The Node ID of the Repository object.
    /// The [ID] represents a unique identifier that is Base64 obfuscated
    ///
    /// [ID]: https://docs.github.com/en/graphql/reference/scalars#id
    id: String,
    /// The repository's name with owner.
    name_with_owner: String,
    /// The description of the repository.
    // FIXME(ytmimi) I eventualy plan to write this value to the database,
    // but I don't need it no.
    #[allow(unused)]
    description: String,
    /// Identifies the date and time when the repository was archived.
    #[serde(with = "time::serde::iso8601::option")]
    // FIXME(ytmimi) I eventualy plan to write this value to the database,
    // but I don't need it no.
    #[allow(unused)]
    archived_at: Option<OffsetDateTime>,
    /// Identifies if the repository is a fork.
    // FIXME(ytmimi) I eventualy plan to write this value to the database,
    // but I don't need it no.
    #[allow(unused)]
    is_fork: bool,
    /// Indicates if the repository has been locked or not.
    // FIXME(ytmimi) I eventualy plan to write this value to the database,
    // but I don't need it no.
    #[allow(unused)]
    is_locked: bool,
    /// Identifies the date and time when the repository was last pushed to.
    #[serde(with = "time::serde::iso8601")]
    pushed_at: OffsetDateTime,
    /// Identifies the date and time when the object was last updated.
    #[serde(with = "time::serde::iso8601")]
    updated_at: OffsetDateTime,
    /// A list containing a breakdown of the language composition of the repository.
    languages: Languages,
    /// The Ref associated with the repository's default branch.
    default_branch_ref: GitBranchRef,
}

impl RepositoryInfo {
    /// Get the GitHub GraphQL ID for this repository
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Get the name of the repostor with the owner included.
    pub fn name_with_owner(&self) -> &str {
        self.name_with_owner.as_str()
    }

    /// Provides an iterator over the languages in this git repository
    pub fn languages<'a>(&'a self) -> LanguageIterator<impl Iterator<Item = &'a LanguageNode>> {
        LanguageIterator {
            total_size: self.languages.total_size,
            inner: self.languages.edges.iter(),
        }
    }

    /// How much of this repository was written in Rust
    pub fn percent_of_code_in_rust(&self) -> f64 {
        self.languages()
            .filter(|programming_language| programming_language.name() == "Rust")
            .next()
            .map_or(0.0, |programming_language| {
                programming_language.percent_of_code_in_repo()
            })
    }

    /// Returns a reference to the latest commit hash fetched from GitHub
    pub fn commit_hash(&self) -> &str {
        &self.default_branch_ref.target.oid
    }

    /// The last time that the repository was pushe
    pub fn pushed_at(&self) -> OffsetDateTime {
        self.pushed_at
    }

    /// The last time that the repository was pushe
    pub fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }
}

/// A list of languages associated with the Repository.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Languages {
    /// Identifies the total number of of languages found in a repository
    #[allow(unused)]
    total_count: usize,
    /// The total size in bytes of the repository
    total_size: usize,
    /// Represents the languages of a repository.
    edges: Vec<LanguageNode>,
}

pub struct LanguageIterator<I> {
    total_size: usize,
    inner: I,
}

pub struct ProgramingLanguage<'a> {
    percent_of_code_in_repo: f64,
    name: &'a str,
}

impl<'a> ProgramingLanguage<'a> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn percent_of_code_in_repo(&self) -> f64 {
        self.percent_of_code_in_repo
    }
}

impl<'a, I> Iterator for LanguageIterator<I>
where
    I: Iterator<Item = &'a LanguageNode>,
{
    type Item = ProgramingLanguage<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|language_node| ProgramingLanguage {
            percent_of_code_in_repo: ((language_node.size as f64) / (self.total_size as f64))
                * 100f64,
            name: &language_node.node.name,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct LanguageNode {
    /// The number of bytes of code written in the language.
    size: usize,
    node: LanguageName,
}

#[derive(Debug, Deserialize)]
struct LanguageName {
    /// The name of the current language.
    name: String,
}

#[derive(Debug, Deserialize)]
struct GitBranchRef {
    target: GitCommit,
}

#[derive(Debug, Deserialize)]
struct GitCommit {
    oid: String,
}
