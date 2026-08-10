use super::options::PrintOptions;
use super::utils::{colorize_tag, format_path, make_clickable_link, make_line_link};
use crate::core::{ScanResult, TodoItem};
use colored::Colorize;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

/// Renders `result` as a hierarchical tree, grouped by file or (if
/// `options.group_by_tag`) by tag.
pub fn print_tree<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    if result.is_empty() {
        writeln!(writer, "{}", "No TODO items found.".dimmed())?;
        return Ok(());
    }

    if options.group_by_tag {
        print_tree_by_tag(writer, result, options)?;
    } else {
        print_tree_by_file(writer, result, options)?;
    }

    Ok(())
}

fn print_tree_by_file<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    let sorted_files = result.sorted_files();
    let total_files = sorted_files.len();

    for (idx, (path, items)) in sorted_files.iter().enumerate() {
        let is_last_file = idx == total_files - 1;
        print_file_header(writer, path, items.len(), is_last_file, options)?;

        let total_items = items.len();
        for (item_idx, item) in items.iter().enumerate() {
            let is_last_item = item_idx == total_items - 1;
            print_tree_item(writer, item, is_last_file, is_last_item, path, options)?;
        }
    }

    Ok(())
}

fn print_tree_by_tag<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    let mut by_tag: HashMap<String, Vec<(PathBuf, TodoItem)>> = HashMap::new();

    for (path, items) in &result.files_map {
        for item in items {
            by_tag
                .entry(item.tag.clone())
                .or_default()
                .push((path.clone(), item.clone()));
        }
    }

    let mut tags: Vec<_> = by_tag.keys().collect();
    tags.sort();

    let total_tags = tags.len();

    for (idx, tag) in tags.iter().enumerate() {
        let is_last_tag = idx == total_tags - 1;
        let items = by_tag.get(*tag).unwrap();

        let prefix = if is_last_tag {
            "└──"
        } else {
            "├──"
        };
        let colored_tag = colorize_tag(tag, options);
        writeln!(writer, "{} {} ({})", prefix, colored_tag, items.len())?;

        let total_items = items.len();
        for (item_idx, (path, item)) in items.iter().enumerate() {
            let is_last_item = item_idx == total_items - 1;
            let tree_prefix = if is_last_tag { "    " } else { "│   " };
            let item_prefix = if is_last_item {
                "└──"
            } else {
                "├──"
            };

            let display_path = format_path(path, options);
            let link = make_clickable_link(path, item.line, options);

            writeln!(
                writer,
                "{}{} {}:{} - {}",
                tree_prefix,
                item_prefix,
                link.unwrap_or_else(|| display_path.to_string()),
                item.line.to_string().cyan(),
                item.message.dimmed()
            )?;
        }
    }

    Ok(())
}

fn print_file_header<W: Write>(
    writer: &mut W,
    path: &Path,
    item_count: usize,
    is_last: bool,
    options: &PrintOptions,
) -> io::Result<()> {
    let prefix = if is_last { "└──" } else { "├──" };
    let display_path = format_path(path, options);
    let link = make_clickable_link(path, 1, options);

    let path_str = link.unwrap_or_else(|| {
        if options.colored {
            display_path.bold().to_string()
        } else {
            display_path.to_string()
        }
    });

    let count_str = format!("({})", item_count);
    let count_display = if options.colored {
        count_str.dimmed().to_string()
    } else {
        count_str
    };

    writeln!(writer, "{} {} {}", prefix, path_str, count_display)?;
    Ok(())
}

fn print_tree_item<W: Write>(
    writer: &mut W,
    item: &TodoItem,
    is_last_file: bool,
    is_last_item: bool,
    path: &Path,
    options: &PrintOptions,
) -> io::Result<()> {
    let tree_prefix = if is_last_file { "    " } else { "│   " };
    let item_prefix = if is_last_item {
        "└──"
    } else {
        "├──"
    };

    let tag = colorize_tag(&item.tag, options);
    let line_num = if options.colored {
        format!("L{}", item.line).cyan().to_string()
    } else {
        format!("L{}", item.line)
    };

    let line_display = if options.clickable_links {
        make_line_link(path, item.line, options).unwrap_or_else(|| line_num.clone())
    } else {
        line_num
    };

    let author_str = item
        .author
        .as_ref()
        .map(|a| format!("({})", a))
        .unwrap_or_default();

    if author_str.is_empty() {
        writeln!(
            writer,
            "{}{} [{}] {}: {}",
            tree_prefix, item_prefix, line_display, tag, item.message
        )?;
    } else {
        let author_display = if options.colored {
            author_str.yellow().to_string()
        } else {
            author_str
        };
        writeln!(
            writer,
            "{}{} [{}] {} {}: {}",
            tree_prefix, item_prefix, line_display, tag, author_display, item.message
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(tag: &str, line: usize, author: Option<&str>) -> TodoItem {
        TodoItem {
            tag: tag.to_string(),
            message: "msg".to_string(),
            line,
            column: 1,
            line_content: None,
            author: author.map(str::to_string),
            priority: crate::core::TodoPriority::from_tag(tag),
        }
    }

    fn options() -> PrintOptions {
        PrintOptions {
            clickable_links: false,
            colored: false,
            ..PrintOptions::default()
        }
    }

    fn two_file_result() -> ScanResult {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(
            PathBuf::from("a.rs"),
            vec![item("TODO", 1, None), item("FIXME", 2, Some("bob"))],
        );
        result.add_file(PathBuf::from("b.rs"), vec![item("NOTE", 1, None)]);
        result
    }

    fn tag_with_multiple_items_result() -> ScanResult {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(
            PathBuf::from("a.rs"),
            vec![item("TODO", 1, None), item("TODO", 2, None)],
        );
        result
    }

    #[test]
    fn print_tree_reports_empty_result() {
        let result = ScanResult::new(PathBuf::from("."));
        let mut buf = Vec::new();

        print_tree(&mut buf, &result, &options()).unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap().trim(),
            "No TODO items found."
        );
    }

    #[test]
    fn print_tree_by_file_groups_and_orders_by_path() {
        let result = two_file_result();
        let mut buf = Vec::new();

        print_tree(&mut buf, &result, &options()).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let a_pos = output.find("a.rs").unwrap();
        let b_pos = output.find("b.rs").unwrap();
        assert!(a_pos < b_pos);
        assert!(output.contains("TODO"));
        assert!(output.contains("FIXME"));
        assert!(output.contains("(bob)"));
    }

    #[test]
    fn print_tree_by_file_colored_variant_with_author() {
        let result = two_file_result();
        let opts = PrintOptions {
            colored: true,
            clickable_links: true,
            ..PrintOptions::default()
        };
        let mut buf = Vec::new();

        print_tree(&mut buf, &result, &opts).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("msg"));
    }

    #[test]
    fn print_tree_by_tag_groups_items_under_tag_headers() {
        let result = two_file_result();
        let opts = PrintOptions {
            group_by_tag: true,
            ..options()
        };
        let mut buf = Vec::new();

        print_tree(&mut buf, &result, &opts).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("FIXME (1)"));
        assert!(output.contains("NOTE (1)"));
        assert!(output.contains("TODO (1)"));
    }

    #[test]
    fn print_tree_by_tag_colored_variant() {
        let result = two_file_result();
        let opts = PrintOptions {
            group_by_tag: true,
            colored: true,
            clickable_links: true,
            ..PrintOptions::default()
        };
        let mut buf = Vec::new();

        print_tree(&mut buf, &result, &opts).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("msg"));
    }

    #[test]
    fn print_tree_by_tag_marks_non_last_items_within_a_tag() {
        let result = tag_with_multiple_items_result();
        let opts = PrintOptions {
            group_by_tag: true,
            ..options()
        };
        let mut buf = Vec::new();

        print_tree(&mut buf, &result, &opts).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // One tag (header uses "└──" since it's the only/last tag), with two
        // items: the first is not-last ("├──"), the second is last ("└──").
        assert_eq!(output.matches("├──").count(), 1);
        assert_eq!(output.matches("└──").count(), 2);
    }
}
