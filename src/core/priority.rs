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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_tag_maps_critical_tags() {
        for tag in ["BUG", "FIXME", "ERROR"] {
            assert_eq!(Priority::from_tag(tag), Priority::Critical);
        }
    }

    #[test]
    fn from_tag_maps_high_tags() {
        for tag in ["HACK", "WARN", "WARNING", "FIX"] {
            assert_eq!(Priority::from_tag(tag), Priority::High);
        }
    }

    #[test]
    fn from_tag_maps_medium_tags() {
        for tag in ["TODO", "WIP", "MAYBE"] {
            assert_eq!(Priority::from_tag(tag), Priority::Medium);
        }
    }

    #[test]
    fn from_tag_maps_low_tags() {
        for tag in ["NOTE", "XXX", "INFO", "DOCS", "PERF", "TEST", "IDEA"] {
            assert_eq!(Priority::from_tag(tag), Priority::Low);
        }
    }

    #[test]
    fn from_tag_is_case_insensitive() {
        assert_eq!(Priority::from_tag("bug"), Priority::Critical);
        assert_eq!(Priority::from_tag("Fixme"), Priority::Critical);
    }

    #[test]
    fn from_tag_defaults_unknown_to_medium() {
        assert_eq!(Priority::from_tag("CUSTOM"), Priority::Medium);
    }

    #[test]
    fn display_name_matches_variant() {
        assert_eq!(Priority::Critical.display_name(), "Critical");
        assert_eq!(Priority::High.display_name(), "High");
        assert_eq!(Priority::Medium.display_name(), "Medium");
        assert_eq!(Priority::Low.display_name(), "Low");
    }

    #[test]
    fn display_uses_display_name() {
        assert_eq!(Priority::Critical.to_string(), "Critical");
        assert_eq!(format!("{}", Priority::Low), "Low");
    }

    #[test]
    fn ordering_ranks_by_severity() {
        assert!(Priority::Low < Priority::Medium);
        assert!(Priority::Medium < Priority::High);
        assert!(Priority::High < Priority::Critical);
    }
}
