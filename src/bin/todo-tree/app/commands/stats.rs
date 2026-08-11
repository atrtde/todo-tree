use super::{is_ci, scan_with_progress, show_progress};
use crate::app::cli;
use color_eyre::eyre::{Result, WrapErr};
use colored::Colorize;
use serde_json::json;
use todo_tree::config::{CliOptions, Config};
use todo_tree::core::TodoPriority;
use todo_tree::display::{format_duration, priority_to_color};
use todo_tree::parser::TodoParser;
use todo_tree::scanner::{ScanOptions, Scanner};

pub fn run(args: cli::StatsArgs, global: &cli::GlobalOptions) -> Result<()> {
    let path = args
        .path
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let path = path
        .canonicalize()
        .wrap_err_with(|| {
            format!(
                "Failed to resolve path: {}. Check that it exists and you have permission to read it.",
                path.display()
            )
        })?;

    let mut config = Config::load_or_default(&path, global.config.as_deref())?;
    config.merge_with_cli(CliOptions {
        tags: args.tags.clone(),
        include: args.include.clone(),
        exclude: args.exclude.clone(),
        json: args.json,
        flat: false,
        no_color: global.no_color,
        ignore_case: args.ignore_case,
        no_require_colon: args.no_require_colon,
    });

    let case_sensitive = !args.ignore_case && !config.ignore_case;
    let require_colon = if args.no_require_colon {
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
        ..Default::default()
    };

    let scanner = Scanner::new(parser, scan_options);
    let result = scan_with_progress(&scanner, &path, show_progress())?;

    let plain = global.no_color || args.plain;
    if args.plain {
        colored::control::set_override(false);
    }

    if args.json || is_ci() {
        let stats = json!({
            "total_items": result.summary.total_count,
            "files_with_todos": result.summary.files_with_todos,
            "files_scanned": result.summary.files_scanned,
            "tag_counts": result.summary.tag_counts,
            "duration_ms": result.summary.duration_ms,
            "items_per_file": if result.summary.files_with_todos > 0 {
                result.summary.total_count as f64 / result.summary.files_with_todos as f64
            } else {
                0.0
            },
        });
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("{}", "TODO Statistics".bold().underline());
        println!();
        println!("  Total items:        {}", result.summary.total_count);
        println!("  Files with TODOs:   {}", result.summary.files_with_todos);
        println!("  Files scanned:      {}", result.summary.files_scanned);
        println!(
            "  Scan time:          {}",
            format_duration(result.summary.duration_ms)
        );

        if result.summary.files_with_todos > 0 {
            let avg = result.summary.total_count as f64 / result.summary.files_with_todos as f64;
            println!("  Avg items per file: {:.2}", avg);
        }

        println!();
        println!("{}", "By Tag:".bold());

        let mut tags: Vec<_> = result.summary.tag_counts.iter().collect();
        tags.sort_by(|a, b| b.1.cmp(a.1));

        for (tag, count) in tags {
            let percentage = if result.summary.total_count > 0 {
                (*count as f64 / result.summary.total_count as f64) * 100.0
            } else {
                0.0
            };

            let bar_width = 20;
            let filled = ((percentage / 100.0) * bar_width as f64) as usize;
            let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);

            if plain {
                println!("  {:<8} {:>4} ({:>5.1}%) {}", tag, count, percentage, bar);
            } else {
                let color = priority_to_color(TodoPriority::from_tag(tag));
                println!(
                    "  {:<8} {:>4} ({:>5.1}%) {}",
                    tag.color(color),
                    count,
                    percentage,
                    bar.dimmed()
                );
            }
        }
    }

    Ok(())
}
