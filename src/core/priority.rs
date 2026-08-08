//! Tag priority levels.

use serde::{Deserialize, Serialize};

/// Severity level derived from a tag, e.g. `BUG` maps to
/// [`Priority::Critical`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Lowest priority: notes, docs, ideas.
    Low,
    /// Medium priority: general TODOs, work in progress.
    Medium,
    /// High priority: hacks, warnings, quick fixes.
    High,
    /// Highest priority: bugs and errors.
    Critical,
}

impl Priority {
    /// Maps a tag name (case-insensitive) to its priority; unrecognized
    /// tags default to [`Priority::Medium`].
    pub fn from_tag(tag: &str) -> Self {
        match tag.to_uppercase().as_str() {
            "BUG" | "FIXME" | "ERROR" => Priority::Critical,
            "HACK" | "WARN" | "WARNING" | "FIX" => Priority::High,
            "TODO" | "WIP" | "MAYBE" => Priority::Medium,
            "NOTE" | "XXX" | "INFO" | "DOCS" | "PERF" | "TEST" | "IDEA" => Priority::Low,
            _ => Priority::Medium,
        }
    }

    /// Human-readable name for the priority.
    pub fn display_name(&self) -> &'static str {
        match self {
            Priority::Critical => "Critical",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
