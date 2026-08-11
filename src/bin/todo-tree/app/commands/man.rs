//! `tt man`: prints a roff man page to stdout, e.g. for
//! `tt man > /usr/local/share/man/man1/tt.1`.

use crate::app::cli::Cli;
use clap::CommandFactory;
use color_eyre::eyre::{Result, WrapErr};

pub fn run() -> Result<()> {
    let mut cmd = Cli::command();
    cmd.set_bin_name(env!("CARGO_BIN_NAME"));
    cmd = cmd.name(env!("CARGO_BIN_NAME"));
    let man = clap_mangen::Man::new(cmd);
    man.render(&mut std::io::stdout())
        .wrap_err("Failed to render man page")
}
