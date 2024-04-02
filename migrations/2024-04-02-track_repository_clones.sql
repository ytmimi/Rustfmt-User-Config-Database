alter table github_repositories add column last_git_cloned_at timestamp with time zone;
create index if not exists last_git_cloned_at_index on github_repositories using btree (last_git_cloned_at);
