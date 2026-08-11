use super::{is_ci, scan_with_progress, show_progress};
use crate::app::cli;
use color_eyre::eyre::{Result, WrapErr};
use std::path::PathBuf;
use todo_tree::config::{CliOptions, Config};
use todo_tree::display::closest_match;
use todo_tree::parser::TodoParser;
use todo_tree::printer::{OutputFormat, PrintOptions, Printer};
use todo_tree::scanner::{ScanOptions, Scanner};

pub fn run(args: cli::ListArgs, global: &cli::GlobalOptions) -> Result<()> {
    let path = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let path = path.canonicalize().wrap_err_with(|| {
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
        flat: true,
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
    let mut result = scan_with_progress(&scanner, &path, show_progress())?;

    if let Some(filter_tag) = &args.filter {
        result = result.filter_by_tag(filter_tag);

        if result.summary.total_count == 0
            && !config
                .tags
                .iter()
                .any(|t| t.eq_ignore_ascii_case(filter_tag))
            && let Some(suggestion) =
                closest_match(filter_tag, config.tags.iter().map(String::as_str))
        {
            eprintln!("No matches for tag '{filter_tag}' (did you mean '{suggestion}'?)");
        }
    }

    let format = if args.json || is_ci() {
        OutputFormat::Json
    } else {
        OutputFormat::Flat
    };

    let colored = !global.no_color && !args.plain;
    let print_options = PrintOptions {
        format,
        colored,
        show_line_numbers: true,
        full_paths: false,
        clickable_links: colored,
        base_path: Some(path),
        show_summary: format != OutputFormat::Json,
        group_by_tag: false,
    };

    let printer = Printer::new(print_options);
    printer.print(&result)?;

    Ok(())
}
