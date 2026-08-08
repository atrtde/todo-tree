//! Aggregate scan counts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
