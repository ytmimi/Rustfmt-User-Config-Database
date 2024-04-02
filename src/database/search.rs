use crate::database::repository::GitRepository;
use crate::Repository;
use sqlx::PgPool;

pub async fn lookup_repositories(db: PgPool, limit: i32) -> anyhow::Result<Vec<Repository>> {
    let query = "select
    github_graphql_id as id,
    repo_name as name_with_owner,
    git_url,
    percent_of_code_in_rust,
    latest_commit as commit_hash,
    is_fork,
    is_locked,
    archived_at,
    pushed_at,
    updated_at
from github_repositories
where can_clone_repo
    and not is_fork
    and not is_locked
    and archived_at is null
order by record_last_updated
limit $1;
";
    let result: Vec<GitRepository> = sqlx::query_as(query).bind(limit).fetch_all(&db).await?;
    Ok(result.into_iter().map(Into::into).collect())
}
