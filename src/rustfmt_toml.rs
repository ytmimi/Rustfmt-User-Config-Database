use anyhow::Context;
use std::path::{Path, PathBuf};

pub struct RustfmtConfig {
    file_path: String,
    toml: toml::Value,
}

impl RustfmtConfig {
    pub fn relative_path(&self) -> &str {
        &self.file_path
    }
    pub fn to_json(&self) -> anyhow::Result<serde_json::Value> {
        serde_json::to_string(&self.toml)
            .and_then(|s| s.parse())
            .with_context(|| format!("failed to convert {} to json", self.file_path))
    }
}

impl std::fmt::Debug for RustfmtConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustfmtConfig")
            .field("file_path", &self.file_path)
            .field("toml", &self.toml)
            .finish()
    }
}

impl<'u> crate::git::ClonedRepo<'u> {
    pub fn find_rustfmt_configs(&self) -> impl Iterator<Item = RustfmtConfig> + '_ {
        let directory_path = self.path();
        let config_files = search_for_rustfmt_config_files(directory_path);
        config_files.into_iter().filter_map(|f| {
            let absolute_file_path = PathBuf::from(&f);
            let relative_path = absolute_file_path
                .strip_prefix(self.path())
                .map_err(|_| tracing::error!("could not find relative path"))
                .ok()?
                .display()
                .to_string();

            let toml = std::fs::read_to_string(&absolute_file_path)
                .map_err(|_| tracing::error!("unable to read toml file {relative_path}"))
                .ok()?
                .parse::<toml::Value>()
                .map_err(|_| tracing::error!("unable to parse {relative_path} as toml"))
                .ok()?;

            Some(RustfmtConfig {
                file_path: relative_path,
                toml,
            })
        })
    }
}

fn search_for_rustfmt_config_files(path: &Path) -> rust_search::Search {
    rust_search::SearchBuilder::default()
        .location(path)
        .search_input("rustfmt")
        .ext("toml")
        .hidden()
        .build()
}
