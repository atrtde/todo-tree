//! Domain types: TODO items, scan results/summaries, priorities, and the
//! built-in tag catalog.

pub mod file_result;
pub mod scan_result;
pub mod scan_summary;
pub mod sort_order;
pub mod tags;
pub mod todo_item;
pub mod todo_priority;

pub use file_result::FileResult;
pub use scan_result::ScanResult;
pub use scan_summary::ScanSummary;
pub use sort_order::SortOrder;
pub use tags::{DEFAULT_TAGS, TagDefinition};
pub use todo_item::TodoItem;
pub use todo_priority::TodoPriority;
