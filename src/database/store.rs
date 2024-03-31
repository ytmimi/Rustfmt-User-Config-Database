use crate::Repository;

use anyhow::Context;
use sqlx::QueryBuilder;
use sqlx::{PgPool, Postgres};

pub async fn store_in_db<R>(db: PgPool, repositories: R) -> anyhow::Result<()>
where
    R: Iterator<Item = Repository>,
{
    // sqlx still has limited support for inserting multiple items.
    // The `QueryBuilder` API seems to be the best way to do it at this point.
    // https://github.com/launchbadge/sqlx/issues/294#issuecomment-1912678387
    let insert_query = r"insert into github_repositories(
    github_graphql_id,
    repo_name,
    git_url,
    is_fork,
    is_locked,
    latest_commit,
    percent_of_code_in_rust,
    archived_at,
    pushed_at,
    updated_at
)
";

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(insert_query);

    query_builder.push_values(repositories, |mut b, repo| {
        b.push_bind(repo.id().to_string())
            .push_bind(repo.name_with_owner().to_string())
            .push_bind(repo.git_url().to_string())
            .push_bind(repo.is_fork())
            .push_bind(repo.is_locked())
            .push_bind(repo.commit_hash().to_string())
            .push_bind(repo.percent_of_code_in_rust())
            .push_bind(repo.archived_at())
            .push_bind(repo.pushed_at())
            .push_bind(repo.updated_at());
    });
    query_builder.push(
        r"
        on conflict on constraint github_repositories_pkey
        do update set
        repo_name = excluded.repo_name,
        git_url = excluded.git_url,
        is_fork = excluded.is_fork,
        is_locked = excluded.is_locked,
        latest_commit = excluded.latest_commit,
        percent_of_code_in_rust = excluded.percent_of_code_in_rust,
        archived_at = excluded.archived_at,
        pushed_at = excluded.pushed_at,
        updated_at = excluded.updated_at,
        record_last_updated = now();",
    );

    query_builder
        .build()
        .execute(&db)
        .await
        .map(|_| ())
        .with_context(|| "Failed to store in the database")
}
