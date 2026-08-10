use super::{is_ci, load_config, sort_results};
use crate::app::cli;
use color_eyre::eyre::{Result, WrapErr};
use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::overrides::{Override, OverrideBuilder};
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use todo_tree::config::CliOptions;
use todo_tree::parser::TodoParser;
use todo_tree::printer::{OutputFormat, PrintOptions, Printer};
use todo_tree::scanner::{ScanOptions, Scanner};

pub fn run(args: cli::WatchArgs, global: &cli::GlobalOptions) -> Result<()> {
    let scan_args = args.scan;
    let path = scan_args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let path = path
        .canonicalize()
        .wrap_err_with(|| format!("Failed to resolve path: {}", path.display()))?;

    let mut config = load_config(&path, global.config.as_deref())?;
    config.merge_with_cli(CliOptions {
        tags: scan_args.tags.clone(),
        include: scan_args.include.clone(),
        exclude: scan_args.exclude.clone(),
        json: scan_args.json,
        flat: scan_args.flat,
        no_color: global.no_color,
        ignore_case: scan_args.ignore_case,
        no_require_colon: scan_args.no_require_colon,
    });

    let case_sensitive = !scan_args.ignore_case && !config.ignore_case;
    let require_colon = if scan_args.no_require_colon {
        false
    } else {
        config.require_colon
    };

    let parser = TodoParser::with_options(
        &config.tags,
        case_sensitive,
        require_colon,
        config.custom_pattern.as_deref(),
    );

    let scan_options = ScanOptions {
        include: config.include.clone(),
        exclude: config.exclude.clone(),
        max_depth: scan_args.depth,
        follow_links: scan_args.follow_links,
        hidden: scan_args.hidden,
        threads: 0,
        respect_gitignore: true,
    };

    // Built once and reused across every re-scan for this `tt watch`
    // session: rebuilding the parser/overrides/matcher per file-change
    // event would undo the point of caching them.
    let filter = EventFilter::build(&path, &scan_options)?;
    let scanner = Scanner::new(parser, scan_options);

    let format = if scan_args.json {
        OutputFormat::Json
    } else if scan_args.flat {
        OutputFormat::Flat
    } else if is_ci() {
        OutputFormat::Json
    } else {
        OutputFormat::Tree
    };

    let print_options = PrintOptions {
        format,
        colored: !global.no_color,
        show_line_numbers: true,
        full_paths: false,
        clickable_links: !global.no_color,
        base_path: Some(path.clone()),
        show_summary: format != OutputFormat::Json,
        group_by_tag: scan_args.group_by_tag,
    };
    let printer = Printer::new(print_options);

    rescan(&scanner, &path, scan_args.sort, &printer)?;

    let (tx, rx) = mpsc::channel::<DebounceEventResult>();
    let mut debouncer = new_debouncer(Duration::from_millis(args.debounce_ms), tx)
        .wrap_err("Failed to start file watcher")?;
    debouncer
        .watcher()
        .watch(&path, RecursiveMode::Recursive)
        .wrap_err_with(|| format!("Failed to watch {}", path.display()))?;

    eprintln!(
        "Watching {} for changes (Ctrl+C to stop)...",
        path.display()
    );

    for result in rx {
        match result {
            Ok(events) => {
                if events.iter().any(|event| filter.is_relevant(&event.path)) {
                    rescan(&scanner, &path, scan_args.sort, &printer)?;
                }
            }
            Err(err) => {
                eprintln!("Watch error: {err}");
            }
        }
    }

    Ok(())
}

fn rescan(scanner: &Scanner, path: &Path, sort: cli::SortOrder, printer: &Printer) -> Result<()> {
    let mut result = scanner.scan(path)?;
    sort_results(&mut result, sort);
    printer.print(&result)?;
    Ok(())
}

/// Filters raw filesystem-change paths through the same `.gitignore` and
/// `--include`/`--exclude` rules [`Scanner`] uses, so a re-scan is only
/// triggered for paths that could actually change scan output (`notify`
/// itself has no concept of `.gitignore`).
struct EventFilter {
    root: PathBuf,
    gitignore: Gitignore,
    overrides: Option<Override>,
    hidden: bool,
}

impl EventFilter {
    fn build(root: &Path, options: &ScanOptions) -> Result<Self> {
        let gitignore = if options.respect_gitignore {
            let mut builder = GitignoreBuilder::new(root);
            builder.add(root.join(".gitignore"));
            builder
                .build()
                .wrap_err("Failed to build .gitignore matcher")?
        } else {
            Gitignore::empty()
        };

        let overrides = if !options.include.is_empty() || !options.exclude.is_empty() {
            let mut builder = OverrideBuilder::new(root);
            for pattern in &options.include {
                builder
                    .add(pattern)
                    .wrap_err_with(|| format!("Invalid include pattern: {}", pattern))?;
            }
            for pattern in &options.exclude {
                let exclude_pattern = format!("!{}", pattern);
                builder
                    .add(&exclude_pattern)
                    .wrap_err_with(|| format!("Invalid exclude pattern: {}", pattern))?;
            }
            Some(builder.build()?)
        } else {
            None
        };

        Ok(Self {
            root: root.to_path_buf(),
            gitignore,
            overrides,
            hidden: options.hidden,
        })
    }

    fn is_relevant(&self, path: &Path) -> bool {
        // `matched_path_or_any_parents` panics on a path outside the
        // matcher's root; fail open (treat as relevant) rather than crash
        // the watch loop over one stray event.
        if !path.starts_with(&self.root) {
            return true;
        }

        if !self.hidden && is_hidden(&self.root, path) {
            return false;
        }

        let is_dir = path.is_dir();

        // `matched_path_or_any_parents` (not `matched`) so a change inside
        // an ignored directory (e.g. `target/`) is caught even though the
        // `.gitignore` rule ("target/") only names the directory itself,
        // not the file changed within it.
        if self
            .gitignore
            .matched_path_or_any_parents(path, is_dir)
            .is_ignore()
        {
            return false;
        }

        if let Some(overrides) = &self.overrides
            && matches!(overrides.matched(path, is_dir), Match::Ignore(_))
        {
            return false;
        }

        true
    }
}

fn is_hidden(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .any(|component| {
            component
                .as_os_str()
                .to_str()
                .is_some_and(|name| name.starts_with('.'))
        })
}
