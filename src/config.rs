//! `.todorc` configuration file discovery, parsing, and merging with CLI
//! overrides.

use crate::core::tags::default_tag_names;
use color_eyre::eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Resolves the global config directory, honoring `XDG_CONFIG_HOME` on all
/// platforms (not just Linux, where `dirs::config_dir` already does this)
/// before falling back to the platform default.
fn config_home() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(dirs::config_dir)
}

/// CLI-provided overrides to merge into a loaded [`Config`].
#[derive(Debug, Clone, Default)]
pub struct CliOptions {
    /// Tags to search for, overriding the config file's list if present.
    pub tags: Option<Vec<String>>,
    /// Include patterns, overriding the config file's list if present.
    pub include: Option<Vec<String>>,
    /// Exclude patterns, appended to the config file's list if present.
    pub exclude: Option<Vec<String>>,
    /// Forces JSON output on.
    pub json: bool,
    /// Forces flat output on.
    pub flat: bool,
    /// Forces colored output off.
    pub no_color: bool,
    /// Forces case-insensitive tag matching on.
    pub ignore_case: bool,
    /// Forces the trailing-colon requirement off.
    pub no_require_colon: bool,
}

/// A loaded (or default) `.todorc` configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Tags to search for.
    pub tags: Vec<String>,
    /// Glob patterns to include.
    pub include: Vec<String>,
    /// Glob patterns to exclude.
    pub exclude: Vec<String>,
    /// Whether to default to JSON output.
    pub json: bool,
    /// Whether to default to flat output.
    pub flat: bool,
    /// Whether to default to uncolored output.
    pub no_color: bool,
    /// An optional custom tag-matching regex, in place of
    /// [`crate::parser::DEFAULT_REGEX`].
    pub custom_pattern: Option<String>,
    /// Whether tag matching is case-insensitive.
    pub ignore_case: bool,
    /// Whether a trailing colon is required after the tag.
    pub require_colon: bool,
}

impl Config {
    /// Builds a config with the default tag set and strict matching
    /// (case-sensitive, colon required).
    pub fn new() -> Self {
        Self {
            tags: default_tag_names(),
            include: Vec::new(),
            exclude: Vec::new(),
            json: false,
            flat: false,
            no_color: false,
            custom_pattern: None,
            ignore_case: false,
            require_colon: true,
        }
    }

    /// Load configuration from a .todorc file
    ///
    /// Searches for configuration files in the following order:
    /// 1. .todorc in the current directory
    /// 2. .todorc.json in the current directory
    /// 3. .todorc.toml in the current directory
    /// 4. Parent directories (recursive)
    /// 5. `$XDG_CONFIG_HOME/todo-tree/config.json` or `config.toml`, falling
    ///    back to the platform config directory if `XDG_CONFIG_HOME` isn't
    ///    set (global config)
    pub fn load(start_path: &Path) -> Result<Option<Self>> {
        let local_configs = [
            start_path.join(".todorc"),
            start_path.join(".todorc.json"),
            start_path.join(".todorc.toml"),
        ];

        for config_path in &local_configs {
            if config_path.exists() {
                return Self::load_from_file(config_path).map(Some);
            }
        }

        if let Some(parent) = start_path.parent()
            && parent != start_path
            && let Ok(Some(config)) = Self::load(parent)
        {
            return Ok(Some(config));
        }

        if let Some(config_dir) = config_home() {
            let global_configs = [
                config_dir.join("todo-tree").join("config.json"),
                config_dir.join("todo-tree").join("config.toml"),
            ];

            for config_path in &global_configs {
                if config_path.exists() {
                    return Self::load_from_file(config_path).map(Some);
                }
            }
        }

        Ok(None)
    }

    /// Loads and parses a specific config file, auto-detecting JSON vs.
    /// TOML from its extension (falling back to JSON-then-TOML for
    /// extensionless files like `.todorc`).
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("Failed to read config file: {}", path.display()))?;

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let parse_result: Result<Self> = if extension == "toml" {
            toml::from_str(&content).map_err(|e| color_eyre::eyre::eyre!(e))
        } else {
            serde_json::from_str(&content)
                .map_err(|e| color_eyre::eyre::eyre!(e))
                .or_else(|_| toml::from_str(&content).map_err(|e| color_eyre::eyre::eyre!(e)))
        };

        parse_result.wrap_err_with(|| format!("Failed to parse config: {}", path.display()))
    }

    /// Merges CLI-provided overrides into this config in place.
    pub fn merge_with_cli(&mut self, cli: CliOptions) {
        if let Some(tags) = cli.tags
            && !tags.is_empty()
        {
            self.tags = tags;
        }

        if let Some(include) = cli.include
            && !include.is_empty()
        {
            self.include = include;
        }

        if let Some(exclude) = cli.exclude
            && !exclude.is_empty()
        {
            self.exclude.extend(exclude);
        }

        if cli.json {
            self.json = true;
        }
        if cli.flat {
            self.flat = true;
        }
        if cli.no_color {
            self.no_color = true;
        }

        if cli.ignore_case {
            self.ignore_case = true;
        }

        if cli.no_require_colon {
            self.require_colon = false;
        }
    }

    /// Writes this config to `path`, choosing TOML or JSON based on its
    /// extension (JSON if unrecognized).
    pub fn save(&self, path: &Path) -> Result<()> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let content = if extension == "toml" {
            toml::to_string_pretty(self)?
        } else {
            serde_json::to_string_pretty(self)?
        };

        std::fs::write(path, content)
            .wrap_err_with(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    // `XDG_CONFIG_HOME` is process-global; serialize every test that touches
    // it so they don't race each other under parallel test execution.
    static XDG_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn temp_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("todo_tree_config_test_{name}_{unique}"))
    }

    #[test]
    fn load_from_file_parses_json() {
        let path = temp_path("json").with_extension("json");
        fs::write(&path, r#"{"tags": ["TODO", "FIXME"], "ignore_case": true}"#).unwrap();

        let config = Config::load_from_file(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(config.tags, vec!["TODO".to_string(), "FIXME".to_string()]);
        assert!(config.ignore_case);
    }

    #[test]
    fn config_home_prefers_xdg_config_home_when_set() {
        let _lock = XDG_ENV_LOCK.lock().unwrap();
        let dir = temp_path("xdg");

        // SAFETY: mutating process env is inherently racy under parallel
        // test execution; the window is kept as narrow as possible and the
        // var is always removed before returning.
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", &dir);
        }
        let resolved = config_home();
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }

        assert_eq!(resolved, Some(dir));
    }

    #[test]
    fn load_from_file_parses_toml() {
        let path = temp_path("toml").with_extension("toml");
        fs::write(&path, "tags = [\"TODO\", \"FIXME\"]\nignore_case = true\n").unwrap();

        let config = Config::load_from_file(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(config.tags, vec!["TODO".to_string(), "FIXME".to_string()]);
        assert!(config.ignore_case);
    }

    #[test]
    fn save_then_load_round_trips_toml() {
        let path = temp_path("roundtrip").with_extension("toml");
        let mut config = Config::new();
        config.tags = vec!["NOTE".to_string()];

        config.save(&path).unwrap();
        let loaded = Config::load_from_file(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(loaded.tags, vec!["NOTE".to_string()]);
    }

    #[test]
    fn load_does_not_recognize_yaml_files() {
        let dir = temp_path("yaml_dir");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(".todorc.yaml"), "tags:\n  - TODO\n").unwrap();

        let result = Config::load(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert!(
            result.is_none() || result.unwrap().tags != vec!["TODO".to_string()],
            ".todorc.yaml must no longer be picked up as a config file"
        );
    }

    #[test]
    fn config_home_falls_back_to_platform_dir_when_xdg_unset() {
        let _lock = XDG_ENV_LOCK.lock().unwrap();
        let previous = std::env::var_os("XDG_CONFIG_HOME");
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        let resolved = config_home();
        unsafe {
            match &previous {
                Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }

        assert_eq!(resolved, dirs::config_dir());
    }

    #[test]
    fn config_home_falls_back_when_xdg_is_empty() {
        let _lock = XDG_ENV_LOCK.lock().unwrap();
        let previous = std::env::var_os("XDG_CONFIG_HOME");
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", "");
        }
        let resolved = config_home();
        unsafe {
            match &previous {
                Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }

        assert_eq!(resolved, dirs::config_dir());
    }

    #[test]
    fn new_returns_strict_defaults() {
        let config = Config::new();
        assert_eq!(config.tags, default_tag_names());
        assert!(config.include.is_empty());
        assert!(config.exclude.is_empty());
        assert!(!config.json);
        assert!(!config.flat);
        assert!(!config.no_color);
        assert!(config.custom_pattern.is_none());
        assert!(!config.ignore_case);
        assert!(config.require_colon);
    }

    #[test]
    fn load_finds_exact_todorc_filename() {
        let dir = temp_path("exact_todorc");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(".todorc"), r#"{"tags": ["NOTE"]}"#).unwrap();

        let config = Config::load(&dir).unwrap().expect("expected a config");
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(config.tags, vec!["NOTE".to_string()]);
    }

    #[test]
    fn load_recurses_into_parent_directories() {
        let dir = temp_path("parent_recursion");
        let child = dir.join("child");
        fs::create_dir_all(&child).unwrap();
        fs::write(dir.join(".todorc.json"), r#"{"tags": ["PARENT"]}"#).unwrap();

        let config = Config::load(&child).unwrap().expect("expected a config");
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(config.tags, vec!["PARENT".to_string()]);
    }

    #[test]
    fn load_from_file_errors_on_missing_file() {
        let path = temp_path("missing").with_extension("json");
        assert!(Config::load_from_file(&path).is_err());
    }

    #[test]
    fn load_from_file_errors_on_unparseable_content() {
        let path = temp_path("garbage").with_extension("toml");
        fs::write(&path, "not: valid { toml or json").unwrap();

        let result = Config::load_from_file(&path);
        let _ = fs::remove_file(&path);

        assert!(result.is_err());
    }

    #[test]
    fn save_writes_json_for_non_toml_extension() {
        let path = temp_path("save_json").with_extension("json");
        let config = Config::new();

        config.save(&path).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert!(content.trim_start().starts_with('{'));
    }

    #[test]
    fn merge_with_cli_applies_every_override() {
        let mut config = Config::new();
        config.exclude = vec!["existing/**".to_string()];

        config.merge_with_cli(CliOptions {
            tags: Some(vec!["CUSTOM".to_string()]),
            include: Some(vec!["*.rs".to_string()]),
            exclude: Some(vec!["extra/**".to_string()]),
            json: true,
            flat: true,
            no_color: true,
            ignore_case: true,
            no_require_colon: true,
        });

        assert_eq!(config.tags, vec!["CUSTOM".to_string()]);
        assert_eq!(config.include, vec!["*.rs".to_string()]);
        assert_eq!(
            config.exclude,
            vec!["existing/**".to_string(), "extra/**".to_string()]
        );
        assert!(config.json);
        assert!(config.flat);
        assert!(config.no_color);
        assert!(config.ignore_case);
        assert!(!config.require_colon);
    }

    #[test]
    fn merge_with_cli_is_a_no_op_with_default_options() {
        let config_before = Config::new();
        let mut config = Config::new();

        config.merge_with_cli(CliOptions::default());

        assert_eq!(config.tags, config_before.tags);
        assert_eq!(config.include, config_before.include);
        assert_eq!(config.exclude, config_before.exclude);
        assert_eq!(config.json, config_before.json);
        assert_eq!(config.flat, config_before.flat);
        assert_eq!(config.no_color, config_before.no_color);
        assert_eq!(config.ignore_case, config_before.ignore_case);
        assert_eq!(config.require_colon, config_before.require_colon);
    }

    #[test]
    fn merge_with_cli_ignores_empty_tag_and_include_overrides() {
        let mut config = Config::new();
        let original_tags = config.tags.clone();

        config.merge_with_cli(CliOptions {
            tags: Some(vec![]),
            include: Some(vec![]),
            exclude: Some(vec![]),
            ..Default::default()
        });

        assert_eq!(config.tags, original_tags);
        assert!(config.include.is_empty());
        assert!(config.exclude.is_empty());
    }
}
