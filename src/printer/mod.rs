//! Formatting a [`crate::core::ScanResult`] as tree, flat, or JSON output.

/// Flat (one-line-per-item) rendering.
pub mod flat;
/// JSON rendering.
pub mod json;
/// [`OutputFormat`] and [`PrintOptions`].
pub mod options;
/// Summary-block rendering.
pub mod summary;
/// Tree (file- or tag-grouped) rendering.
pub mod tree;
/// Path formatting, terminal hyperlink, and tag-coloring helpers.
pub mod utils;

use crate::core::ScanResult;
use flat::print_flat;
use json::print_json;
pub use options::{OutputFormat, PrintOptions};
use std::io::{self, Write};
use summary::print_summary;
use tree::print_tree;

/// Renders a [`ScanResult`] according to a fixed set of [`PrintOptions`].
pub struct Printer {
    options: PrintOptions,
}

impl Printer {
    /// Creates a printer with the given options. If `options.colored` is
    /// `false`, this also disables the process-wide `colored` crate
    /// override.
    pub fn new(options: PrintOptions) -> Self {
        if !options.colored {
            colored::control::set_override(false);
        }
        Self { options }
    }

    /// Renders `result` to stdout.
    pub fn print(&self, result: &ScanResult) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        self.print_to(&mut handle, result)
    }

    /// Renders `result` to `writer`.
    pub fn print_to<W: Write>(&self, writer: &mut W, result: &ScanResult) -> io::Result<()> {
        match self.options.format {
            OutputFormat::Tree => print_tree(writer, result, &self.options)?,
            OutputFormat::Flat => print_flat(writer, result, &self.options)?,
            OutputFormat::Json => print_json(writer, result, &self.options)?,
        }

        if self.options.show_summary && self.options.format != OutputFormat::Json {
            writeln!(writer)?;
            print_summary(writer, result, &self.options)?;
        }

        Ok(())
    }
}
