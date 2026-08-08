//! JSON rendering.

use super::options::PrintOptions;
use crate::core::ScanResult;
use serde::Serialize;
use std::collections::HashMap;
use std::io::{self, Write};

/// The top-level JSON document written by [`print_json`].
#[derive(Debug, Serialize)]
pub struct JsonOutput {
    /// Files with at least one TODO item, sorted by path.
    pub files: Vec<JsonFileEntry>,
    /// Aggregate counts for the scan.
    pub summary: JsonSummary,
}

/// A single file's TODO items in the JSON output.
#[derive(Debug, Serialize)]
pub struct JsonFileEntry {
    /// The file's display path.
    pub path: String,
    /// The TODO items found in the file.
    pub items: Vec<JsonTodoItem>,
}

/// A single TODO item in the JSON output.
#[derive(Debug, Serialize)]
pub struct JsonTodoItem {
    /// The tag that matched.
    pub tag: String,
    /// The comment text following the tag.
    pub message: String,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column.
    pub column: usize,
    /// The optional author/assignee.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// The item's priority, formatted with `{:?}` (e.g. `"Critical"`).
    pub priority: String,
}

/// Aggregate counts in the JSON output.
#[derive(Debug, Serialize)]
pub struct JsonSummary {
    /// Total number of TODO items found across all files.
    pub total_count: usize,
    /// Number of files that contained at least one TODO item.
    pub files_with_todos: usize,
    /// Total number of files scanned.
    pub files_scanned: usize,
    /// Number of items found per tag.
    pub tag_counts: HashMap<String, usize>,
    /// How long the scan took, in milliseconds.
    pub duration_ms: u128,
}

/// Renders `result` as pretty-printed JSON.
pub fn print_json<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    let json_result = JsonOutput::from_scan_result(result, options);
    let json_str = serde_json::to_string_pretty(&json_result).map_err(io::Error::other)?;
    writeln!(writer, "{}", json_str)?;
    Ok(())
}

impl JsonOutput {
    /// Builds the JSON document from a scan result and print options
    /// (options control whether paths are absolute, relative, or
    /// stripped to `options.base_path`).
    pub fn from_scan_result(result: &ScanResult, options: &PrintOptions) -> Self {
        let mut files: Vec<JsonFileEntry> = result
            .sorted_files()
            .iter()
            .map(|(path, items)| {
                let display_path = if options.full_paths {
                    path.display().to_string()
                } else if let Some(base) = &options.base_path {
                    path.strip_prefix(base)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| path.display().to_string())
                } else {
                    path.display().to_string()
                };

                JsonFileEntry {
                    path: display_path,
                    items: items
                        .iter()
                        .map(|item| JsonTodoItem {
                            tag: item.tag.clone(),
                            message: item.message.clone(),
                            line: item.line,
                            column: item.column,
                            author: item.author.clone(),
                            priority: format!("{:?}", item.priority),
                        })
                        .collect(),
                }
            })
            .collect();

        files.sort_by(|a, b| a.path.cmp(&b.path));

        let summary = JsonSummary {
            total_count: result.summary.total_count,
            files_with_todos: result.summary.files_with_todos,
            files_scanned: result.summary.files_scanned,
            tag_counts: result.summary.tag_counts.clone(),
            duration_ms: result.summary.duration_ms,
        };

        Self { files, summary }
    }
}
