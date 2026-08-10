use super::options::PrintOptions;
use super::utils::{colorize_tag, format_duration};
use crate::core::ScanResult;
use colored::Colorize;
use std::io::{self, Write};

/// Renders the summary block: total/file/scan counts, duration, and a
/// per-tag breakdown.
pub fn print_summary<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    let summary_line = format!(
        "Found {} TODO items in {} files ({} files scanned in {})",
        result.summary.total_count,
        result.summary.files_with_todos,
        result.summary.files_scanned,
        format_duration(result.summary.duration_ms)
    );

    if options.colored {
        writeln!(writer, "{}", summary_line.bold())?;
    } else {
        writeln!(writer, "{}", summary_line)?;
    }

    if !result.summary.tag_counts.is_empty() {
        let mut tags: Vec<_> = result.summary.tag_counts.iter().collect();
        tags.sort_by(|a, b| b.1.cmp(a.1));

        let breakdown: Vec<String> = tags
            .iter()
            .map(|(tag, count)| {
                if options.colored {
                    format!("{}: {}", colorize_tag(tag, options), count)
                } else {
                    format!("{}: {}", tag, count)
                }
            })
            .collect();

        writeln!(writer, "  {}", breakdown.join(", "))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TodoItem;
    use std::path::PathBuf;

    fn item(tag: &str) -> TodoItem {
        TodoItem {
            tag: tag.to_string(),
            message: "msg".to_string(),
            line: 1,
            column: 1,
            line_content: None,
            author: None,
            priority: crate::core::TodoPriority::from_tag(tag),
        }
    }

    #[test]
    fn print_summary_uncolored_reports_counts() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(PathBuf::from("a.rs"), vec![item("TODO"), item("FIXME")]);

        let opts = PrintOptions {
            colored: false,
            ..PrintOptions::default()
        };
        let mut buf = Vec::new();
        print_summary(&mut buf, &result, &opts).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Found 2 TODO items in 1 files"));
        assert!(output.contains("TODO: 1"));
        assert!(output.contains("FIXME: 1"));
    }

    #[test]
    fn print_summary_colored_variant() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(PathBuf::from("a.rs"), vec![item("TODO")]);

        let opts = PrintOptions {
            colored: true,
            ..PrintOptions::default()
        };
        let mut buf = Vec::new();
        print_summary(&mut buf, &result, &opts).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Found 1 TODO items"));
        assert!(output.contains("TODO"));
    }

    #[test]
    fn print_summary_omits_breakdown_line_when_no_tags() {
        let result = ScanResult::new(PathBuf::from("."));
        let opts = PrintOptions::default();
        let mut buf = Vec::new();

        print_summary(&mut buf, &result, &opts).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert_eq!(output.lines().count(), 1);
    }
}
