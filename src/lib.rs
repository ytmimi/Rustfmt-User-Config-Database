pub mod cli;
mod database;
mod git;
mod github;
mod rustfmt_toml;

pub use database::search::lookup_repositories;
pub use database::store::{store_in_db, store_rustfmt_configs};
pub use github::{GitHubRepoSearch, ProgrammingLanguage, RepoSearchResults, Repository};
