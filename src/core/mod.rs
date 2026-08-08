//! Domain types: TODO items, scan results/summaries, priorities, and the
//! built-in tag catalog.

pub mod priority;
pub mod tags;
pub mod types;

pub use priority::Priority;
pub use tags::{DEFAULT_TAGS, TagDefinition};
pub use types::{FileResult, ScanResult, ScanSummary, TodoItem};
