//! Core library for `todo-tree`: parses TODO-style comments out of source
//! files and produces structured scan results.
//!
//! This crate is intentionally silent and side-effect-free: it never prints
//! to stdout/stderr and never reads CLI arguments. Those concerns live in
//! the `todo-tree`/`tt` binaries, which are thin CLI wrappers around
//! [`parser::TodoParser`], [`scanner::Scanner`], and [`printer::Printer`].

#![warn(missing_docs)]

/// `.todorc` configuration file discovery, parsing, and merging with CLI
/// overrides.
pub mod config;
/// Domain types shared across the crate: TODO items, scan results and
/// summaries, priorities, and the built-in tag catalog.
pub mod core;
/// Regex-based parsing of TODO-style comments out of file content.
pub mod parser;
/// Formatting a [`core::ScanResult`] as tree, flat, or JSON output.
pub mod printer;
/// Directory walking and file parsing orchestration.
pub mod scanner;

pub use crate::core::{Priority, ScanResult, ScanSummary, TodoItem};
