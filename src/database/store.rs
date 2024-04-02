use crate::git::ClonedRepo;
use crate::Repository;
use std::ops::DerefMut;

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

pub async fn store_rustfmt_configs<'a, R>(db: &PgPool, repositories: R) -> anyhow::Result<()>
where
    R: Iterator<Item = (&'a Repository, ClonedRepo<'a>)>,
{
    let insert_query = r"insert into rustfmt_configuration_files(
    github_graphql_id,
    latest_commit,
    file_path,
    config
)
values
";

    let update_query = r"update github_repositories
set last_git_cloned_at = now()
where github_graphql_id in (
";

    let mut transaction = db.begin().await?;

    let mut repos = repositories.peekable();
    let mut insert_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(insert_query);
    let mut update_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(update_query);

    while let Some((repo, cloned_repo)) = repos.next() {
        update_query_builder.push_bind(repo.id());
        if repos.peek().is_some() {
            update_query_builder.push(", ");
        } else {
            update_query_builder.push(")");
        }

        let Some(commit) = cloned_repo.head() else {
            tracing::error!(
                "could not find latest commit for {}",
                repo.name_with_owner()
            );
            continue;
        };

        let mut configs = cloned_repo.find_rustfmt_configs().peekable();

        while let Some(config) = configs.next() {
            let Ok(json) = config.to_json() else {
                tracing::error!(
                    "Could not convert TOML to JSON for {}",
                    repo.name_with_owner()
                );
                continue;
            };
            insert_query_builder.push("(");
            insert_query_builder.push_bind(repo.id());
            insert_query_builder.push(", ");
            insert_query_builder.push_bind(commit.clone());
            insert_query_builder.push(", ");
            insert_query_builder.push_bind(config.relative_path().to_owned());
            insert_query_builder.push(", ");
            insert_query_builder.push_bind(json);
            insert_query_builder.push(")");

            if configs.peek().is_some() || repos.peek().is_some() {
                insert_query_builder.push(",\n");
            }
        }
    }
    insert_query_builder.push(
        r"
on conflict on constraint rustfmt_configuration_files_pkey
do update set
latest_commit = excluded.latest_commit,
config = excluded.config,
record_last_updated = now();",
    );

    tracing::debug!("{}", insert_query_builder.sql());
    tracing::debug!("{}", update_query_builder.sql());

    if let Err(e) = insert_query_builder
        .build()
        .execute(transaction.deref_mut())
        .await
    {
        tracing::error!("Failed to store rustfmt.toml files in the database: {e}");
        return transaction
            .rollback()
            .await
            .context("Failed to rollback transaction");
    };

    if let Err(e) = update_query_builder
        .build()
        .execute(transaction.deref_mut())
        .await
    {
        tracing::error!("Failed to store rustfmt.toml files in the database: {e}");
        return transaction
            .rollback()
            .await
            .context("Failed to rollback transaction");
    };

    transaction
        .commit()
        .await
        .context("Failed to commit transaction")
}
