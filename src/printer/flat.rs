use super::options::PrintOptions;
use super::utils::{colorize_tag, format_path, make_clickable_link};
use crate::core::{ScanResult, TodoItem};
use colored::Colorize;
use std::io::{self, Write};
use std::path::Path;

/// Renders `result` as a flat, one-line-per-item list sorted by file then
/// line number.
pub fn print_flat<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    if result.is_empty() {
        writeln!(writer, "{}", "No TODO items found.".dimmed())?;
        return Ok(());
    }

    let mut all_items = result.all_items();
    all_items.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.line.cmp(&b.1.line)));

    for (path, item) in all_items {
        print_flat_item(writer, &path, &item, options)?;
    }

    Ok(())
}

fn print_flat_item<W: Write>(
    writer: &mut W,
    path: &Path,
    item: &TodoItem,
    options: &PrintOptions,
) -> io::Result<()> {
    let display_path = format_path(path, options);
    let link = make_clickable_link(path, item.line, options);

    let path_str = link.unwrap_or_else(|| {
        if options.colored {
            display_path.bold().to_string()
        } else {
            display_path.to_string()
        }
    });

    let line_col = format!(":{}:{}", item.line, item.column);
    let line_col_display = if options.colored {
        line_col.cyan().to_string()
    } else {
        line_col
    };

    let tag = colorize_tag(&item.tag, options);

    writeln!(
        writer,
        "{}{} [{}] {}",
        path_str, line_col_display, tag, item.message
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn item(tag: &str, line: usize, message: &str) -> TodoItem {
        TodoItem {
            tag: tag.to_string(),
            message: message.to_string(),
            line,
            column: 1,
            line_content: None,
            author: None,
            priority: crate::core::Priority::from_tag(tag),
        }
    }

    fn options() -> PrintOptions {
        PrintOptions {
            clickable_links: false,
            colored: false,
            ..PrintOptions::default()
        }
    }

    #[test]
    fn print_flat_reports_empty_result() {
        let result = ScanResult::new(PathBuf::from("."));
        let mut buf = Vec::new();

        print_flat(&mut buf, &result, &options()).unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap().trim(),
            "No TODO items found."
        );
    }

    #[test]
    fn print_flat_sorts_by_file_then_line() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(PathBuf::from("b.rs"), vec![item("TODO", 5, "second file")]);
        result.add_file(
            PathBuf::from("a.rs"),
            vec![
                item("FIXME", 2, "later line"),
                item("TODO", 1, "first line"),
            ],
        );
        let mut buf = Vec::new();

        print_flat(&mut buf, &result, &options()).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("a.rs") && lines[0].contains("first line"));
        assert!(lines[1].contains("a.rs") && lines[1].contains("later line"));
        assert!(lines[2].contains("b.rs") && lines[2].contains("second file"));
    }

    #[test]
    fn print_flat_colored_variant_includes_tag_and_message() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(PathBuf::from("a.rs"), vec![item("TODO", 1, "hello")]);
        let opts = PrintOptions {
            colored: true,
            clickable_links: false,
            ..PrintOptions::default()
        };
        let mut buf = Vec::new();

        print_flat(&mut buf, &result, &opts).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("TODO"));
        assert!(output.contains("hello"));
    }
}
