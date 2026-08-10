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

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(total_count: usize, files_with_todos: usize) -> ScanSummary {
        ScanSummary {
            total_count,
            files_with_todos,
            files_scanned: files_with_todos + 1,
            tag_counts: HashMap::new(),
            duration_ms: 0,
        }
    }

    #[test]
    fn avg_items_per_file_divides_when_files_present() {
        assert_eq!(summary(10, 4).avg_items_per_file(), 2.5);
    }

    #[test]
    fn avg_items_per_file_is_zero_when_no_files_with_todos() {
        assert_eq!(summary(0, 0).avg_items_per_file(), 0.0);
    }

    #[test]
    fn tag_percentage_divides_when_total_present() {
        assert_eq!(summary(4, 1).tag_percentage(1), 25.0);
    }

    #[test]
    fn tag_percentage_is_zero_when_total_is_zero() {
        assert_eq!(summary(0, 0).tag_percentage(0), 0.0);
    }
}
