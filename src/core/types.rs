//! Scan result and TODO item types.

use super::priority::Priority;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A single matched TODO-style comment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoItem {
    /// The tag that matched, e.g. `"TODO"` or `"FIXME"`.
    pub tag: String,
    /// The comment text following the tag.
    pub message: String,
    /// 1-based line number the tag was found on.
    pub line: usize,
    /// 1-based column the tag starts at.
    pub column: usize,
    /// The full source line the tag was found on, if captured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_content: Option<String>,
    /// The optional author/assignee captured from `TAG(name): ...` syntax.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// The priority derived from the tag.
    pub priority: Priority,
}

impl TodoItem {
    /// Formats the author as `"(name)"`, or an empty string if none was
    /// captured.
    pub fn format_author(&self) -> String {
        self.author
            .as_ref()
            .map(|a| format!("({})", a))
            .unwrap_or_default()
    }
}

/// TODO items found in a single file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileResult {
    /// The file's path, as a display string.
    pub path: String,
    /// The TODO items found in the file.
    pub items: Vec<TodoItem>,
}

/// Aggregate counts for a scan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScanSummary {
    /// Total number of TODO items found across all files.
    pub total_count: usize,
    /// Number of files that contained at least one TODO item.
    pub files_with_todos: usize,
    /// Total number of files scanned.
    pub files_scanned: usize,
    /// Number of items found per tag.
    pub tag_counts: HashMap<String, usize>,
    /// How long the scan took, in milliseconds.
    #[serde(default)]
    pub duration_ms: u128,
}

impl ScanSummary {
    /// Average number of TODO items per file with at least one TODO item;
    /// `0.0` if none were found.
    pub fn avg_items_per_file(&self) -> f64 {
        if self.files_with_todos > 0 {
            self.total_count as f64 / self.files_with_todos as f64
        } else {
            0.0
        }
    }

    /// Percentage `count` represents out of `total_count`; `0.0` if
    /// `total_count` is zero.
    pub fn tag_percentage(&self, count: usize) -> f64 {
        if self.total_count > 0 {
            (count as f64 / self.total_count as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// The result of scanning a directory tree for TODO items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Populated only for the JSON-deserialized form (see
    /// [`ScanResult::to_json_format`]); `None` for a live in-memory scan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileResult>>,
    /// The live in-memory form: TODO items keyed by file path.
    #[serde(skip)]
    pub files_map: HashMap<PathBuf, Vec<TodoItem>>,
    /// Aggregate counts for this scan.
    pub summary: ScanSummary,
    /// The root directory that was scanned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<PathBuf>,
}

impl ScanResult {
    /// Creates an empty result rooted at `root`.
    pub fn new(root: PathBuf) -> Self {
        Self {
            files: None,
            files_map: HashMap::new(),
            summary: ScanSummary {
                total_count: 0,
                files_with_todos: 0,
                files_scanned: 0,
                tag_counts: HashMap::new(),
                duration_ms: 0,
            },
            root: Some(root),
        }
    }

    /// Reconstructs a result from its JSON-deserialized form.
    pub fn from_json(files: Vec<FileResult>, summary: ScanSummary) -> Self {
        Self {
            files: Some(files),
            files_map: HashMap::new(),
            summary,
            root: None,
        }
    }

    /// Whether the scan found no TODO items.
    pub fn is_empty(&self) -> bool {
        if let Some(files) = &self.files {
            files.is_empty()
        } else {
            self.files_map.is_empty()
        }
    }

    /// Records a scanned file's TODO items, updating the summary counts.
    /// Files with no items are counted as scanned but not stored.
    pub fn add_file(&mut self, path: PathBuf, items: Vec<TodoItem>) {
        self.summary.files_scanned += 1;

        if !items.is_empty() {
            self.summary.files_with_todos += 1;
            self.summary.total_count += items.len();

            for item in &items {
                *self.summary.tag_counts.entry(item.tag.clone()).or_insert(0) += 1;
            }

            self.files_map.insert(path, items);
        }
    }

    /// Flattens the result into `(file path, item)` pairs.
    pub fn all_items(&self) -> Vec<(PathBuf, TodoItem)> {
        let mut items = Vec::new();
        for (path, file_items) in &self.files_map {
            for item in file_items {
                items.push((path.clone(), item.clone()));
            }
        }
        items
    }

    /// Files with their items, sorted by path.
    pub fn sorted_files(&self) -> Vec<(&PathBuf, &Vec<TodoItem>)> {
        let mut files: Vec<_> = self.files_map.iter().collect();
        files.sort_by(|a, b| a.0.cmp(b.0));
        files
    }

    /// Returns a new result containing only items whose tag matches `tag`
    /// (case-insensitive).
    pub fn filter_by_tag(&self, tag: &str) -> ScanResult {
        let root = self.root.clone().unwrap_or_else(|| PathBuf::from("."));
        let mut result = ScanResult::new(root);
        result.summary.files_scanned = self.summary.files_scanned;
        result.summary.duration_ms = self.summary.duration_ms;

        for (path, items) in &self.files_map {
            let filtered: Vec<TodoItem> = items
                .iter()
                .filter(|item| item.tag.eq_ignore_ascii_case(tag))
                .cloned()
                .collect();

            if !filtered.is_empty() {
                result.add_file(path.clone(), filtered);
            }
        }

        result
    }

    /// Converts to the JSON-serializable form (populates `files`, clears
    /// `files_map`).
    pub fn to_json_format(&self) -> Self {
        let mut files: Vec<FileResult> = self
            .files_map
            .iter()
            .map(|(path, items)| FileResult {
                path: path.display().to_string(),
                items: items.clone(),
            })
            .collect();

        files.sort_by(|a, b| a.path.cmp(&b.path));

        Self {
            files: Some(files),
            files_map: HashMap::new(),
            summary: self.summary.clone(),
            root: None,
        }
    }

    /// The scan's files, regardless of which internal form they're
    /// currently stored in.
    pub fn get_files(&self) -> Vec<FileResult> {
        if let Some(files) = &self.files {
            files.clone()
        } else {
            self.to_json_format().files.unwrap_or_default()
        }
    }
}
