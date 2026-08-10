use crate::app::cli;
use color_eyre::eyre::Result;
use std::path::Path;
use todo_tree::config::Config;
use todo_tree::core::ScanResult;

pub mod init;
pub mod list;
pub mod scan;
pub mod stats;
pub mod tags;
pub mod watch;
pub mod workflow;

pub(crate) fn load_config(path: &Path, config_path: Option<&Path>) -> Result<Config> {
    if let Some(config_path) = config_path {
        return Config::load_from_file(config_path);
    }

    match Config::load(path)? {
        Some(config) => Ok(config),
        None => Ok(Config::new()),
    }
}

pub(crate) fn save_config(config: &Config) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_files = [
        current_dir.join(".todorc"),
        current_dir.join(".todorc.json"),
        current_dir.join(".todorc.toml"),
    ];

    for path in &config_files {
        if path.exists() {
            return config.save(path);
        }
    }

    let path = current_dir.join(".todorc.json");
    config.save(&path)
}

/// Detects a CI environment via the conventional `CI` env var, set by
/// GitHub Actions, GitLab CI, CircleCI, Travis CI, and most other
/// providers. An explicit `--json`/`--flat` flag always takes precedence
/// over this; it only decides the *default* output format when the user
/// hasn't picked one, so CI logs come out machine-readable by default.
pub(crate) fn is_ci() -> bool {
    match std::env::var("CI") {
        Ok(value) => !value.is_empty() && value != "0" && !value.eq_ignore_ascii_case("false"),
        Err(_) => false,
    }
}

pub(crate) fn sort_results(result: &mut ScanResult, sort: cli::SortOrder) {
    match sort {
        cli::SortOrder::File => {}
        cli::SortOrder::Line => {
            for items in result.files_map.values_mut() {
                items.sort_by_key(|item| item.line);
            }
        }
        cli::SortOrder::Priority => {
            for items in result.files_map.values_mut() {
                items.sort_by_key(|item| std::cmp::Reverse(item.priority));
            }
        }
    }
}
