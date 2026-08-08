//! The built-in tag catalog.

use super::priority::Priority;

/// A single recognized tag: its name, human-readable description, and
/// default priority.
#[derive(Debug, Clone, PartialEq)]
pub struct TagDefinition {
    /// The tag's name, e.g. `"TODO"`.
    pub name: &'static str,
    /// A short human-readable description of what the tag means.
    pub description: &'static str,
    /// The tag's default priority.
    pub priority: Priority,
}

/// The default set of recognized tags, grouped by priority.
pub const DEFAULT_TAGS: &[TagDefinition] = &[
    // Medium
    TagDefinition {
        name: "TODO",
        description: "General TODO items",
        priority: Priority::Medium,
    },
    TagDefinition {
        name: "WIP",
        description: "Work in progress",
        priority: Priority::Medium,
    },
    TagDefinition {
        name: "MAYBE",
        description: "Potential future work",
        priority: Priority::Medium,
    },
    // Critical
    TagDefinition {
        name: "FIXME",
        description: "Items that need fixing",
        priority: Priority::Critical,
    },
    TagDefinition {
        name: "BUG",
        description: "Known bugs",
        priority: Priority::Critical,
    },
    TagDefinition {
        name: "ERROR",
        description: "Error handling needed",
        priority: Priority::Critical,
    },
    // High
    TagDefinition {
        name: "HACK",
        description: "Hacky solutions",
        priority: Priority::High,
    },
    TagDefinition {
        name: "WARN",
        description: "Warnings",
        priority: Priority::High,
    },
    TagDefinition {
        name: "WARNING",
        description: "Warning about potential issues",
        priority: Priority::High,
    },
    TagDefinition {
        name: "FIX",
        description: "Quick fix needed",
        priority: Priority::High,
    },
    // Low priority
    TagDefinition {
        name: "NOTE",
        description: "Notes and documentation",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "XXX",
        description: "Items requiring attention",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "INFO",
        description: "Informational notes",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "DOCS",
        description: "Documentation needed",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "PERF",
        description: "Performance issues",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "TEST",
        description: "Test-related items",
        priority: Priority::Low,
    },
    TagDefinition {
        name: "IDEA",
        description: "Ideas for future consideration",
        priority: Priority::Low,
    },
];

/// The names of [`DEFAULT_TAGS`], in order.
pub fn default_tag_names() -> Vec<String> {
    DEFAULT_TAGS.iter().map(|t| t.name.to_string()).collect()
}

/// Looks up a tag definition by name (case-insensitive).
pub fn find_tag(name: &str) -> Option<&'static TagDefinition> {
    DEFAULT_TAGS
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tag_names_matches_default_tags() {
        let names = default_tag_names();
        assert_eq!(names.len(), DEFAULT_TAGS.len());
        assert_eq!(names[0], "TODO");
        assert!(names.contains(&"FIXME".to_string()));
    }

    #[test]
    fn find_tag_is_case_insensitive() {
        let tag = find_tag("todo").expect("TODO should be found");
        assert_eq!(tag.name, "TODO");
        assert_eq!(tag.priority, Priority::Medium);
    }

    #[test]
    fn find_tag_returns_none_for_unknown_tag() {
        assert!(find_tag("NOT_A_REAL_TAG").is_none());
    }
}
