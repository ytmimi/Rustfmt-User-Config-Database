mod graphql;
mod search;
use graphql::RepositoryInfo as GraphQLRepoInfo;

pub use graphql::ProgrammingLanguage;
pub use search::{GitHubRepoSearch, RepoSearchResults};

pub struct Repository {
    inner: RepositoryInner,
}

impl From<GraphQLRepoInfo> for Repository {
    fn from(value: GraphQLRepoInfo) -> Self {
        Repository {
            inner: RepositoryInner::GitHub(value),
        }
    }
}

enum RepositoryInner {
    GitHub(GraphQLRepoInfo),
}

impl Repository {
    pub fn id(&self) -> &str {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.id(),
        }
    }

    pub fn name_with_owner(&self) -> &str {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.name_with_owner(),
        }
    }

    pub fn url(&self) -> &str {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.url(),
        }
    }

    pub fn git_url(&self) -> &str {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.git_url(),
        }
    }

    pub fn percent_of_code_in_rust(&self) -> f64 {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.percent_of_code_in_rust(),
        }
    }

    pub fn commit_hash(&self) -> &str {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.commit_hash(),
        }
    }

    pub fn pushed_at(&self) -> time::OffsetDateTime {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.pushed_at(),
        }
    }

    pub fn updated_at(&self) -> time::OffsetDateTime {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.updated_at(),
        }
    }
}
