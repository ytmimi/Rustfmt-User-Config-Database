mod graphql;
mod search;
use graphql::RepositoryInfo as GraphQLRepoInfo;
use std::fmt::{Debug, Display};

use crate::database::repository::GitRepository;
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

impl From<GitRepository> for Repository {
    fn from(value: GitRepository) -> Self {
        Repository {
            inner: RepositoryInner::Databae(value),
        }
    }
}

enum RepositoryInner {
    GitHub(GraphQLRepoInfo),
    Databae(GitRepository),
}

impl Repository {
    pub fn id(&self) -> &str {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.id(),
            RepositoryInner::Databae(repo) => repo.id(),
        }
    }

    pub fn name_with_owner(&self) -> &str {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.name_with_owner(),
            RepositoryInner::Databae(repo) => repo.name_with_owner(),
        }
    }

    pub fn url(&self) -> &str {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.url(),
            RepositoryInner::Databae(repo) => repo.url(),
        }
    }

    pub fn git_url(&self) -> &str {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.git_url(),
            RepositoryInner::Databae(repo) => repo.git_url(),
        }
    }

    pub fn percent_of_code_in_rust(&self) -> f64 {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.percent_of_code_in_rust(),
            RepositoryInner::Databae(repo) => repo.percent_of_code_in_rust(),
        }
    }

    pub fn commit_hash(&self) -> &str {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.commit_hash(),
            RepositoryInner::Databae(repo) => repo.commit_hash(),
        }
    }

    pub fn is_fork(&self) -> bool {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.is_fork(),
            RepositoryInner::Databae(repo) => repo.is_fork(),
        }
    }

    pub fn is_locked(&self) -> bool {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.is_locked(),
            RepositoryInner::Databae(repo) => repo.is_locked(),
        }
    }

    pub fn archived_at(&self) -> Option<time::OffsetDateTime> {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.archived_at(),
            RepositoryInner::Databae(repo) => repo.archived_at(),
        }
    }

    pub fn pushed_at(&self) -> time::OffsetDateTime {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.pushed_at(),
            RepositoryInner::Databae(repo) => repo.pushed_at(),
        }
    }

    pub fn updated_at(&self) -> time::OffsetDateTime {
        match &self.inner {
            RepositoryInner::GitHub(repo) => repo.updated_at(),
            RepositoryInner::Databae(repo) => repo.updated_at(),
        }
    }
}

impl Display for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repository")
            .field("name", &self.name_with_owner())
            .field("commit", &self.commit_hash())
            .field("url", &self.url())
            .field(
                "percent_of_code_in_rust",
                &format_args!("{:.2}", self.percent_of_code_in_rust()),
            )
            .finish()
    }
}

impl Debug for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repository")
            .field("id", &self.id())
            .field("name_with_owner", &self.name_with_owner())
            .field("commit_hash", &self.commit_hash())
            .field("url", &self.url())
            .field("git_url", &self.git_url())
            .field(
                "percent_of_code_in_rust",
                &format_args!("{:.2}", self.percent_of_code_in_rust()),
            )
            .field("is_fork", &self.is_fork())
            .field("is_locked", &self.is_locked())
            .field("archived_at", &self.archived_at())
            .field("pushed_at", &self.pushed_at())
            .field("updated_at", &self.updated_at())
            .finish()
    }
}
