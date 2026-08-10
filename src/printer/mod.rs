//! Formatting a [`crate::core::ScanResult`] as tree, flat, or JSON output.

/// Flat (one-line-per-item) rendering.
pub mod flat;
/// JSON rendering.
pub mod json;
/// [`OutputFormat`] and [`PrintOptions`].
pub mod options;
/// Summary-block rendering.
pub mod summary;
/// Tree (file- or tag-grouped) rendering.
pub mod tree;
/// Path formatting, terminal hyperlink, and tag-coloring helpers.
pub mod utils;

use crate::core::ScanResult;
use flat::print_flat;
use json::print_json;
pub use options::{OutputFormat, PrintOptions};
use std::io::{self, Write};
use summary::print_summary;
use tree::print_tree;

/// Renders a [`ScanResult`] according to a fixed set of [`PrintOptions`].
pub struct Printer {
    options: PrintOptions,
}

impl Printer {
    /// Creates a printer with the given options. If `options.colored` is
    /// `false`, this also disables the process-wide `colored` crate
    /// override.
    pub fn new(options: PrintOptions) -> Self {
        if !options.colored {
            colored::control::set_override(false);
        }
        Self { options }
    }

    /// Renders `result` to stdout.
    pub fn print(&self, result: &ScanResult) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        self.print_to(&mut handle, result)
    }

    /// Renders `result` to `writer`.
    pub fn print_to<W: Write>(&self, writer: &mut W, result: &ScanResult) -> io::Result<()> {
        match self.options.format {
            OutputFormat::Tree => print_tree(writer, result, &self.options)?,
            OutputFormat::Flat => print_flat(writer, result, &self.options)?,
            OutputFormat::Json => print_json(writer, result, &self.options)?,
        }

        if self.options.show_summary && self.options.format != OutputFormat::Json {
            writeln!(writer)?;
            print_summary(writer, result, &self.options)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TodoItem;
    use std::path::PathBuf;

    fn result_with_one_item() -> ScanResult {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(
            PathBuf::from("a.rs"),
            vec![TodoItem {
                tag: "TODO".to_string(),
                message: "msg".to_string(),
                line: 1,
                column: 1,
                line_content: None,
                author: None,
                priority: crate::core::TodoPriority::Medium,
            }],
        );
        result
    }

    fn opts(format: OutputFormat, show_summary: bool) -> PrintOptions {
        PrintOptions {
            format,
            colored: false,
            clickable_links: false,
            show_summary,
            ..PrintOptions::default()
        }
    }

    #[test]
    fn print_to_renders_tree_with_summary() {
        let printer = Printer::new(opts(OutputFormat::Tree, true));
        let mut buf = Vec::new();
        printer.print_to(&mut buf, &result_with_one_item()).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("a.rs"));
        assert!(output.contains("Found 1 TODO items"));
    }

    #[test]
    fn print_to_renders_flat_without_summary() {
        let printer = Printer::new(opts(OutputFormat::Flat, false));
        let mut buf = Vec::new();
        printer.print_to(&mut buf, &result_with_one_item()).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("a.rs"));
        assert!(!output.contains("Found"));
    }

    #[test]
    fn print_to_renders_json_and_never_appends_summary() {
        let printer = Printer::new(opts(OutputFormat::Json, true));
        let mut buf = Vec::new();
        printer.print_to(&mut buf, &result_with_one_item()).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.trim_start().starts_with('{'));
        assert!(!output.contains("Found"));
    }

    #[test]
    fn new_disables_global_color_override_when_uncolored() {
        let _printer = Printer::new(opts(OutputFormat::Tree, false));
        assert!(!colored::control::SHOULD_COLORIZE.should_colorize());
        colored::control::unset_override();
    }

    #[test]
    fn print_writes_to_stdout_without_error() {
        let printer = Printer::new(opts(OutputFormat::Flat, false));
        printer.print(&result_with_one_item()).unwrap();
    }
}
