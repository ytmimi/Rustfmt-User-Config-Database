#[derive(sqlx::FromRow)]
pub(crate) struct GitRepository {
    id: String,
    name_with_owner: String,
    git_url: String,
    percent_of_code_in_rust: f64,
    commit_hash: String,
    is_fork: bool,
    is_locked: bool,
    archived_at: Option<time::OffsetDateTime>,
    pushed_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}
impl GitRepository {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn name_with_owner(&self) -> &str {
        &self.name_with_owner
    }

    pub(crate) fn url(&self) -> &str {
        // Trim `.git` from the end of the url
        &self.git_url[..self.git_url.len() - 4]
    }

    pub(crate) fn git_url(&self) -> &str {
        &self.git_url
    }

    pub(crate) fn percent_of_code_in_rust(&self) -> f64 {
        self.percent_of_code_in_rust
    }

    pub(crate) fn commit_hash(&self) -> &str {
        &self.commit_hash
    }

    pub(crate) fn is_fork(&self) -> bool {
        self.is_fork
    }

    pub(crate) fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub(crate) fn archived_at(&self) -> Option<time::OffsetDateTime> {
        self.archived_at
    }

    pub(crate) fn pushed_at(&self) -> time::OffsetDateTime {
        self.pushed_at
    }

    pub(crate) fn updated_at(&self) -> time::OffsetDateTime {
        self.updated_at
    }
}
