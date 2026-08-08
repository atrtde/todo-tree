//! Domain types: TODO items, scan results/summaries, priorities, and the
//! built-in tag catalog.

pub mod file_result;
pub mod priority;
pub mod scan_result;
pub mod summary;
pub mod tags;
pub mod todo_item;

pub use file_result::FileResult;
pub use priority::Priority;
pub use scan_result::ScanResult;
pub use summary::ScanSummary;
pub use tags::{DEFAULT_TAGS, TagDefinition};
pub use todo_item::TodoItem;
