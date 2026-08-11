use crate::app::cli::{self, ConfigFormat};
use color_eyre::eyre::{Result, eyre};
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use todo_tree::config::Config;

pub fn run(args: cli::InitArgs, global: &cli::GlobalOptions) -> Result<()> {
    let filename = match args.format {
        ConfigFormat::Json => ".todorc.json",
        ConfigFormat::Toml => ".todorc.toml",
    };

    let path = PathBuf::from(filename);

    if path.exists() && !args.force && !confirm_overwrite(filename, global.no_input)? {
        return Ok(());
    }

    let config = Config::new();
    config.save(&path)?;

    println!("Created configuration file: {}", filename);
    println!("\nYou can customize the following settings:");
    println!("  - tags: List of tags to search for");
    println!("  - include: File patterns to include");
    println!("  - exclude: File patterns to exclude");
    println!("  - json: Default to JSON output");
    println!("  - flat: Default to flat output");

    Ok(())
}

/// Asks the user whether to overwrite `filename`, returning `Ok(true)` to
/// proceed. Only prompts when stdin is a TTY and `--no-input` wasn't given;
/// otherwise fails with an actionable error rather than hanging on a read
/// that will never get input (e.g. in CI or a pipeline).
fn confirm_overwrite(filename: &str, no_input: bool) -> Result<bool> {
    if no_input || !std::io::stdin().is_terminal() {
        return Err(eyre!(
            "Config file {} already exists. Use --force to overwrite.",
            filename
        ));
    }

    print!("Config file {} already exists. Overwrite? [y/N] ", filename);
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        Ok(true)
    } else {
        println!("Aborted.");
        Ok(false)
    }
}
