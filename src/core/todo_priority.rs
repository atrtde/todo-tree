//! Tag priority levels.

use serde::{Deserialize, Serialize};

/// Severity level derived from a tag, e.g. `BUG` maps to
/// [`TodoPriority::Critical`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TodoPriority {
    /// Lowest priority: notes, docs, ideas.
    Low,
    /// Medium priority: general TODOs, work in progress.
    Medium,
    /// High priority: hacks, warnings, quick fixes.
    High,
    /// Highest priority: bugs and errors.
    Critical,
}

impl TodoPriority {
    /// Maps a tag name (case-insensitive) to its priority; unrecognized
    /// tags default to [`TodoPriority::Medium`].
    pub fn from_tag(tag: &str) -> Self {
        match tag.to_uppercase().as_str() {
            "BUG" | "FIXME" | "ERROR" => TodoPriority::Critical,
            "HACK" | "WARN" | "WARNING" | "FIX" => TodoPriority::High,
            "TODO" | "WIP" | "MAYBE" => TodoPriority::Medium,
            "NOTE" | "XXX" | "INFO" | "DOCS" | "PERF" | "TEST" | "IDEA" => TodoPriority::Low,
            _ => TodoPriority::Medium,
        }
    }

    /// Human-readable name for the priority.
    pub fn display_name(&self) -> &'static str {
        match self {
            TodoPriority::Critical => "Critical",
            TodoPriority::High => "High",
            TodoPriority::Medium => "Medium",
            TodoPriority::Low => "Low",
        }
    }
}

impl std::fmt::Display for TodoPriority {
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
            assert_eq!(TodoPriority::from_tag(tag), TodoPriority::Critical);
        }
    }

    #[test]
    fn from_tag_maps_high_tags() {
        for tag in ["HACK", "WARN", "WARNING", "FIX"] {
            assert_eq!(TodoPriority::from_tag(tag), TodoPriority::High);
        }
    }

    #[test]
    fn from_tag_maps_medium_tags() {
        for tag in ["TODO", "WIP", "MAYBE"] {
            assert_eq!(TodoPriority::from_tag(tag), TodoPriority::Medium);
        }
    }

    #[test]
    fn from_tag_maps_low_tags() {
        for tag in ["NOTE", "XXX", "INFO", "DOCS", "PERF", "TEST", "IDEA"] {
            assert_eq!(TodoPriority::from_tag(tag), TodoPriority::Low);
        }
    }

    #[test]
    fn from_tag_is_case_insensitive() {
        assert_eq!(TodoPriority::from_tag("bug"), TodoPriority::Critical);
        assert_eq!(TodoPriority::from_tag("Fixme"), TodoPriority::Critical);
    }

    #[test]
    fn from_tag_defaults_unknown_to_medium() {
        assert_eq!(TodoPriority::from_tag("CUSTOM"), TodoPriority::Medium);
    }

    #[test]
    fn display_name_matches_variant() {
        assert_eq!(TodoPriority::Critical.display_name(), "Critical");
        assert_eq!(TodoPriority::High.display_name(), "High");
        assert_eq!(TodoPriority::Medium.display_name(), "Medium");
        assert_eq!(TodoPriority::Low.display_name(), "Low");
    }

    #[test]
    fn display_uses_display_name() {
        assert_eq!(TodoPriority::Critical.to_string(), "Critical");
        assert_eq!(format!("{}", TodoPriority::Low), "Low");
    }

    #[test]
    fn ordering_ranks_by_severity() {
        assert!(TodoPriority::Low < TodoPriority::Medium);
        assert!(TodoPriority::Medium < TodoPriority::High);
        assert!(TodoPriority::High < TodoPriority::Critical);
    }
}
