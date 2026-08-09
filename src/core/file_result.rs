//! TODO items found in a single file.

use super::todo_item::TodoItem;
use serde::{Deserialize, Serialize};

/// TODO items found in a single file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileResult {
    /// The file's path, as a display string.
    pub path: String,
    /// The TODO items found in the file.
    pub items: Vec<TodoItem>,
}
