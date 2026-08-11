//! `tt completions <shell>`: prints a shell completion script to stdout.

use crate::app::cli::{self, Cli};
use clap::CommandFactory;
use color_eyre::eyre::Result;

pub fn run(args: cli::CompletionsArgs) -> Result<()> {
    let mut cmd = Cli::command();
    // `CARGO_BIN_NAME` resolves per binary target at compile time (unlike
    // `Cli`'s fixed `#[command(name = "todo-tree")]`), so `tt`'s completions
    // are generated under the `tt` name and `todo-tree`'s under its own.
    clap_complete::generate(
        args.shell,
        &mut cmd,
        env!("CARGO_BIN_NAME"),
        &mut std::io::stdout(),
    );
    Ok(())
}
