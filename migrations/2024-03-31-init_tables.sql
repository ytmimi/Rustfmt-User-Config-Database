create table if not exists github_repositories (
    github_graphql_id text primary key not null,
    repo_name text not null,
    git_url text not null,
    is_fork bool not null default false,
    is_locked bool not null default false,
    latest_commit text not null,
    percent_of_code_in_rust float default 0,
    archived_at timestamp with time zone,
    pushed_at timestamp with time zone not null,
    updated_at timestamp with time zone not null,
    record_last_updated timestamp with time zone not null default now(),
    can_clone_repo bool not null default true
);
create index if not exists repo_name_index on github_repositories using btree (repo_name);
create index if not exists record_last_updated_index on github_repositories using btree (record_last_updated);

create table if not exists rustfmt_configuration_files (
    github_graphql_id text not null,
    latest_commit text not null,
    file_path text not null,
    record_last_updated timestamp with time zone default now(),
    config jsonb,
    primary key(github_graphql_id, file_path),
    constraint fk_github_graphql_id foreign key(github_graphql_id) references github_repositories(github_graphql_id) on delete cascade
);
create index if not exists rustfmt_config_file_path_index on rustfmt_configuration_files using btree (file_path);
create index if not exists rustfmt_config_gin_index on rustfmt_configuration_files using gin (config);
