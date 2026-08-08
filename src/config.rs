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

#[derive(Debug, Clone, Default)]
pub struct CliOptions {
    pub tags: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub json: bool,
    pub flat: bool,
    pub no_color: bool,
    pub ignore_case: bool,
    pub no_require_colon: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub tags: Vec<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub json: bool,
    pub flat: bool,
    pub no_color: bool,
    pub custom_pattern: Option<String>,
    pub ignore_case: bool,
    pub require_colon: bool,
}

impl Config {
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
}
