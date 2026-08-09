use crate::app::cli::{self, ConfigFormat};
use color_eyre::eyre::{Result, eyre};
use std::path::PathBuf;
use todo_tree::config::Config;

pub fn run(args: cli::InitArgs) -> Result<()> {
    let filename = match args.format {
        ConfigFormat::Json => ".todorc.json",
        ConfigFormat::Toml => ".todorc.toml",
    };

    let path = PathBuf::from(filename);

    if path.exists() && !args.force {
        return Err(eyre!(
            "Config file {} already exists. Use --force to overwrite.",
            filename
        ));
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
