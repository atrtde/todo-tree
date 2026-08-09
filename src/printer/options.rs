//! Output format and print option types.

use std::path::PathBuf;

/// The output format a [`super::Printer`] renders a
/// [`crate::core::ScanResult`] as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Hierarchical, file-grouped tree view.
    Tree,
    /// One line per item, no grouping.
    Flat,
    /// Machine-readable JSON.
    Json,
}

/// Options controlling how a scan result is rendered.
#[derive(Debug, Clone)]
pub struct PrintOptions {
    /// The output format to render.
    pub format: OutputFormat,
    /// Whether to colorize output.
    pub colored: bool,
    /// Whether to show line numbers.
    pub show_line_numbers: bool,
    /// Whether to show full (vs. relative) file paths.
    pub full_paths: bool,
    /// Whether to emit clickable OSC 8 terminal hyperlinks.
    pub clickable_links: bool,
    /// The root path scanned, used to compute relative paths and links.
    pub base_path: Option<PathBuf>,
    /// Whether to print the summary block after the main output.
    pub show_summary: bool,
    /// Whether to group items by tag instead of by file.
    pub group_by_tag: bool,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Tree,
            colored: true,
            show_line_numbers: true,
            full_paths: false,
            clickable_links: true,
            base_path: None,
            show_summary: true,
            group_by_tag: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_use_tree_format_and_color() {
        let options = PrintOptions::default();
        assert_eq!(options.format, OutputFormat::Tree);
        assert!(options.colored);
        assert!(options.show_line_numbers);
        assert!(!options.full_paths);
        assert!(options.clickable_links);
        assert!(options.base_path.is_none());
        assert!(options.show_summary);
        assert!(!options.group_by_tag);
    }

    #[test]
    fn output_format_equality() {
        assert_eq!(OutputFormat::Tree, OutputFormat::Tree);
        assert_ne!(OutputFormat::Tree, OutputFormat::Json);
    }
}
