//! Directory walking and file parsing orchestration.

use crate::core::{ScanResult, TodoItem};
use crate::parser::TodoParser;
use color_eyre::eyre::{Result, WrapErr};
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use std::path::Path;
use std::time::Instant;

/// Options controlling how [`Scanner`] walks a directory tree.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Glob patterns to include; empty means "include everything not
    /// excluded".
    pub include: Vec<String>,
    /// Glob patterns to exclude.
    pub exclude: Vec<String>,
    /// Maximum directory depth to descend; `0` means unlimited.
    pub max_depth: usize,
    /// Whether to follow symlinks.
    pub follow_links: bool,
    /// Whether to include hidden files and directories.
    pub hidden: bool,
    /// Number of worker threads to use; `0` lets the walker choose.
    pub threads: usize,
    /// Whether to respect `.gitignore`/global/local git ignore rules.
    pub respect_gitignore: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            max_depth: 0,
            follow_links: false,
            hidden: false,
            threads: 0,
            respect_gitignore: true,
        }
    }
}

/// Walks a directory tree, parsing every file with a [`TodoParser`].
pub struct Scanner {
    parser: TodoParser,
    options: ScanOptions,
}

impl Scanner {
    /// Creates a scanner using `parser` and `options`.
    pub fn new(parser: TodoParser, options: ScanOptions) -> Self {
        Self { parser, options }
    }

    /// Walks `root`, parsing every matching file and collecting the
    /// results.
    pub fn scan(&self, root: &Path) -> Result<ScanResult> {
        let start = Instant::now();
        let root = root
            .canonicalize()
            .wrap_err_with(|| format!("Failed to resolve path: {}", root.display()))?;

        let mut result = ScanResult::new(root.clone());
        let mut builder = WalkBuilder::new(&root);

        builder
            .hidden(!self.options.hidden)
            .follow_links(self.options.follow_links)
            .git_ignore(self.options.respect_gitignore)
            .git_global(self.options.respect_gitignore)
            .git_exclude(self.options.respect_gitignore);

        if self.options.max_depth > 0 {
            builder.max_depth(Some(self.options.max_depth));
        }

        if self.options.threads > 0 {
            builder.threads(self.options.threads);
        }

        if !self.options.include.is_empty() || !self.options.exclude.is_empty() {
            let mut override_builder = OverrideBuilder::new(&root);
            for pattern in &self.options.include {
                override_builder
                    .add(pattern)
                    .wrap_err_with(|| format!("Invalid include pattern: {}", pattern))?;
            }

            for pattern in &self.options.exclude {
                let exclude_pattern = format!("!{}", pattern);
                override_builder
                    .add(&exclude_pattern)
                    .wrap_err_with(|| format!("Invalid exclude pattern: {}", pattern))?;
            }

            let overrides = override_builder.build()?;
            builder.overrides(overrides);
        }

        for entry in builder.build() {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    if path.is_dir() {
                        continue;
                    }

                    if let Some(file_type) = entry.file_type()
                        && !file_type.is_file()
                    {
                        continue;
                    }

                    match self.parse_file(path) {
                        Ok(items) => {
                            result.add_file(path.to_path_buf(), items);
                        }
                        Err(_) => {
                            result.summary.files_scanned += 1;
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }

        result.summary.duration_ms = start.elapsed().as_millis();

        Ok(result)
    }

    fn parse_file(&self, path: &Path) -> Result<Vec<TodoItem>> {
        self.parser
            .parse_file(path)
            .wrap_err_with(|| format!("Failed to parse file: {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::TodoParser;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("todo_tree_scanner_test_{name}_{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn parser() -> TodoParser {
        TodoParser::new(&["TODO".to_string(), "FIXME".to_string()], true)
    }

    fn scanner(options: ScanOptions) -> Scanner {
        Scanner::new(parser(), options)
    }

    #[test]
    fn default_options_respect_gitignore_and_no_limits() {
        let options = ScanOptions::default();
        assert!(options.respect_gitignore);
        assert_eq!(options.max_depth, 0);
        assert_eq!(options.threads, 0);
        assert!(!options.hidden);
        assert!(!options.follow_links);
        assert!(options.include.is_empty());
        assert!(options.exclude.is_empty());
    }

    #[test]
    fn scan_finds_todos_and_counts_all_files() {
        let dir = temp_dir("basic");
        fs::write(dir.join("a.rs"), "// TODO: fix this\nfn main() {}\n").unwrap();
        fs::write(dir.join("b.rs"), "fn main() {}\n").unwrap();

        let result = scanner(ScanOptions::default()).scan(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(result.summary.total_count, 1);
        assert_eq!(result.summary.files_with_todos, 1);
        assert_eq!(result.summary.files_scanned, 2);
    }

    #[test]
    fn scan_errors_on_nonexistent_path() {
        let dir =
            std::env::temp_dir().join("todo_tree_scanner_test_missing_dir_definitely_not_here");
        let result = scanner(ScanOptions::default()).scan(&dir);
        assert!(result.is_err());
    }

    #[test]
    fn scan_respects_include_patterns() {
        let dir = temp_dir("include");
        fs::write(dir.join("a.rs"), "// TODO: rust file\n").unwrap();
        fs::write(dir.join("b.py"), "# TODO: python file\n").unwrap();

        let options = ScanOptions {
            include: vec!["*.rs".to_string()],
            ..Default::default()
        };
        let result = scanner(options).scan(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(result.summary.total_count, 1);
    }

    #[test]
    fn scan_respects_exclude_patterns() {
        let dir = temp_dir("exclude");
        fs::write(dir.join("a.rs"), "// TODO: keep\n").unwrap();
        fs::write(dir.join("b.rs"), "// TODO: drop\n").unwrap();

        let options = ScanOptions {
            exclude: vec!["b.rs".to_string()],
            ..Default::default()
        };
        let result = scanner(options).scan(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(result.summary.total_count, 1);
    }

    #[test]
    fn scan_errors_on_invalid_include_pattern() {
        let dir = temp_dir("bad_pattern");

        let options = ScanOptions {
            include: vec!["[".to_string()],
            ..Default::default()
        };
        let result = scanner(options).scan(&dir);
        let _ = fs::remove_dir_all(&dir);

        assert!(result.is_err());
    }

    #[test]
    fn scan_skips_hidden_files_by_default() {
        let dir = temp_dir("hidden");
        fs::write(dir.join(".hidden.rs"), "// TODO: hidden\n").unwrap();

        let result = scanner(ScanOptions::default()).scan(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(result.summary.total_count, 0);
    }

    #[test]
    fn scan_includes_hidden_files_when_enabled() {
        let dir = temp_dir("hidden_enabled");
        fs::write(dir.join(".hidden.rs"), "// TODO: hidden\n").unwrap();

        let options = ScanOptions {
            hidden: true,
            ..Default::default()
        };
        let result = scanner(options).scan(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(result.summary.total_count, 1);
    }

    #[test]
    fn scan_respects_max_depth() {
        let dir = temp_dir("depth");
        let nested = dir.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.join("top.rs"), "// TODO: top\n").unwrap();
        fs::write(nested.join("deep.rs"), "// TODO: deep\n").unwrap();

        let options = ScanOptions {
            max_depth: 1,
            ..Default::default()
        };
        let result = scanner(options).scan(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(result.summary.total_count, 1);
    }

    #[test]
    fn scan_counts_unparseable_files_as_scanned() {
        let dir = temp_dir("bad_utf8");
        fs::write(dir.join("bad.rs"), [0xFF, 0xFE, 0xFD]).unwrap();

        let result = scanner(ScanOptions::default()).scan(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(result.summary.files_scanned, 1);
        assert_eq!(result.summary.total_count, 0);
    }

    #[test]
    fn scan_uses_custom_thread_count() {
        let dir = temp_dir("threads");
        fs::write(dir.join("a.rs"), "// TODO: threaded\n").unwrap();

        let options = ScanOptions {
            threads: 2,
            ..Default::default()
        };
        let result = scanner(options).scan(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(result.summary.total_count, 1);
    }
}
