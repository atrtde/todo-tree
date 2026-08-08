//! CLI entry point: parses arguments and dispatches to command handlers.

pub(crate) mod cli;
pub(crate) mod commands;
pub(crate) mod display;

use clap::Parser;
use cli::{Cli, Command};
use color_eyre::eyre::Result;

/// Parse CLI args, install error reporting, and dispatch to the matching
/// command handler.
pub fn run() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    if cli.global.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    match cli.get_command() {
        Command::Scan(args) => commands::scan::run(args, &cli.global),
        Command::List(args) => commands::list::run(args, &cli.global),
        Command::Tags(args) => commands::tags::run(args, &cli.global),
        Command::Init(args) => commands::init::run(args),
        Command::Workflow(args) => commands::workflow::run(args),
        Command::Stats(args) => commands::stats::run(args, &cli.global),
    }
}
