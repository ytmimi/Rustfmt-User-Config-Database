use anyhow::Context;
use git2::build::RepoBuilder;
use git2::{FetchOptions, Repository as GitRepository};
use std::path::{Path, PathBuf};

pub fn clone_repo<'u, 'd>(
    url: &'u str,
    directory: &'d Path,
    depth: i32,
) -> anyhow::Result<ClonedRepo<'u>> {
    if !directory.is_dir() {
        return Err(anyhow::anyhow!(
            "{} is not a direcotry",
            directory.display()
        ));
    }

    let mut options = FetchOptions::new();
    options
        .depth(depth)
        .prune(git2::FetchPrune::Off)
        .update_fetchhead(false)
        .download_tags(git2::AutotagOption::None);

    let mut repo_builder = RepoBuilder::new();
    repo_builder
        .fetch_options(options)
        .clone(url, directory)
        .map_err(|e| {
            tracing::error!("Git Error Code: {:?}", e.code());
            e
        })
        .map(|repo| ClonedRepo {
            repo,
            url,
            directory: directory.to_owned(),
        })
        .with_context(|| format!("failed to clone repo: {url} to {}", directory.display()))
}

pub struct ClonedRepo<'url> {
    repo: GitRepository,
    url: &'url str,
    directory: PathBuf,
}

impl<'url> std::fmt::Debug for ClonedRepo<'url> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClonedRepo")
            .field("url", &self.url)
            .field("directory", &self.directory.display())
            .finish()
    }
}

impl<'url> ClonedRepo<'url> {
    pub(crate) fn path(&self) -> &Path {
        &self.directory
    }
}
