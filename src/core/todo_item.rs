//! A single matched TODO-style comment.

use super::priority::Priority;
use serde::{Deserialize, Serialize};

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
