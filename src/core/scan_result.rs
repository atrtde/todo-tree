//! The result of scanning a directory tree.

use super::file_result::FileResult;
use super::summary::ScanSummary;
use super::todo_item::TodoItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Priority;

    fn item(tag: &str, line: usize) -> TodoItem {
        TodoItem {
            tag: tag.to_string(),
            message: "msg".to_string(),
            line,
            column: 1,
            line_content: None,
            author: None,
            priority: Priority::from_tag(tag),
        }
    }

    #[test]
    fn new_is_empty() {
        let result = ScanResult::new(PathBuf::from("/tmp"));
        assert!(result.is_empty());
        assert_eq!(result.root, Some(PathBuf::from("/tmp")));
        assert!(result.files.is_none());
    }

    #[test]
    fn add_file_updates_summary_and_map() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(
            PathBuf::from("a.rs"),
            vec![item("TODO", 1), item("FIXME", 2)],
        );

        assert!(!result.is_empty());
        assert_eq!(result.summary.files_scanned, 1);
        assert_eq!(result.summary.files_with_todos, 1);
        assert_eq!(result.summary.total_count, 2);
        assert_eq!(result.summary.tag_counts.get("TODO"), Some(&1));
        assert_eq!(result.summary.tag_counts.get("FIXME"), Some(&1));
    }

    #[test]
    fn add_file_with_no_items_counts_scanned_but_not_stored() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(PathBuf::from("empty.rs"), vec![]);

        assert_eq!(result.summary.files_scanned, 1);
        assert_eq!(result.summary.files_with_todos, 0);
        assert!(result.files_map.is_empty());
    }

    #[test]
    fn all_items_flattens_files_map() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(PathBuf::from("a.rs"), vec![item("TODO", 1)]);
        result.add_file(PathBuf::from("b.rs"), vec![item("FIXME", 2)]);

        let items = result.all_items();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn sorted_files_orders_by_path() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(PathBuf::from("z.rs"), vec![item("TODO", 1)]);
        result.add_file(PathBuf::from("a.rs"), vec![item("TODO", 1)]);

        let files = result.sorted_files();
        assert_eq!(files[0].0, &PathBuf::from("a.rs"));
        assert_eq!(files[1].0, &PathBuf::from("z.rs"));
    }

    #[test]
    fn filter_by_tag_keeps_only_matching_items_case_insensitively() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(
            PathBuf::from("a.rs"),
            vec![item("TODO", 1), item("FIXME", 2)],
        );

        let filtered = result.filter_by_tag("todo");
        assert_eq!(filtered.summary.total_count, 1);
        assert_eq!(filtered.all_items()[0].1.tag, "TODO");
    }

    #[test]
    fn filter_by_tag_drops_files_with_no_matches() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(PathBuf::from("a.rs"), vec![item("FIXME", 1)]);

        let filtered = result.filter_by_tag("todo");
        assert!(filtered.is_empty());
    }

    #[test]
    fn to_json_format_populates_files_and_clears_map() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(PathBuf::from("a.rs"), vec![item("TODO", 1)]);

        let json = result.to_json_format();
        assert!(json.files_map.is_empty());
        assert_eq!(json.files.as_ref().unwrap().len(), 1);
        assert_eq!(json.files.as_ref().unwrap()[0].path, "a.rs");
    }

    #[test]
    fn from_json_round_trips_into_files_form() {
        let files = vec![FileResult {
            path: "a.rs".to_string(),
            items: vec![item("TODO", 1)],
        }];
        let summary = ScanSummary {
            total_count: 1,
            files_with_todos: 1,
            files_scanned: 1,
            tag_counts: HashMap::new(),
            duration_ms: 5,
        };

        let result = ScanResult::from_json(files, summary);
        assert!(result.root.is_none());
        assert_eq!(result.get_files().len(), 1);
    }

    #[test]
    fn get_files_uses_live_map_when_files_is_none() {
        let mut result = ScanResult::new(PathBuf::from("."));
        result.add_file(PathBuf::from("a.rs"), vec![item("TODO", 1)]);

        assert_eq!(result.get_files().len(), 1);
    }

    #[test]
    fn get_files_uses_files_directly_when_present() {
        let files = vec![FileResult {
            path: "a.rs".to_string(),
            items: vec![],
        }];
        let summary = ScanSummary {
            total_count: 0,
            files_with_todos: 0,
            files_scanned: 0,
            tag_counts: HashMap::new(),
            duration_ms: 0,
        };
        let result = ScanResult::from_json(files, summary);

        assert_eq!(result.get_files().len(), 1);
    }
}
