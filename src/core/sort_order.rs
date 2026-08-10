//! Result ordering for [`super::ScanResult`].

/// How [`super::ScanResult::sort_by`] orders items within each file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortOrder {
    /// Leaves each file's items in scan order (no sorting applied).
    #[default]
    File,
    /// Sort by line number.
    Line,
    /// Sort by tag priority, highest first.
    Priority,
}
