use crate::app::cli;
use color_eyre::eyre::Result;
use todo_tree::config::Config;
use todo_tree::core::TodoPriority;
use todo_tree::core::tags::default_tag_names;
use todo_tree::display::{closest_match, priority_to_color};

pub fn run(args: cli::TagsArgs, global: &cli::GlobalOptions) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let mut config = Config::load_or_default(&current_dir, global.config.as_deref())?;

    if let Some(new_tag) = &args.add {
        if !config.tags.iter().any(|t| t.eq_ignore_ascii_case(new_tag)) {
            config.tags.push(new_tag.to_uppercase());
            config.save_in_cwd()?;
            println!("Added tag: {}", new_tag.to_uppercase());
        } else {
            println!("Tag already exists: {}", new_tag);
        }
        return Ok(());
    }

    if let Some(remove_tag) = &args.remove {
        let original_len = config.tags.len();
        config.tags.retain(|t| !t.eq_ignore_ascii_case(remove_tag));
        if config.tags.len() < original_len {
            config.save_in_cwd()?;
            println!("Removed tag: {}", remove_tag);
        } else {
            let suggestion =
                closest_match(remove_tag, config.tags.iter().map(String::as_str));
            match suggestion {
                Some(suggestion) => {
                    println!("Tag not found: {remove_tag} (did you mean {suggestion}?)")
                }
                None => println!("Tag not found: {remove_tag}"),
            }
        }
        return Ok(());
    }

    if args.reset {
        config.tags = default_tag_names();
        config.save_in_cwd()?;
        println!("Tags reset to defaults");
        return Ok(());
    }

    if args.json {
        let json = serde_json::json!({
            "tags": config.tags,
            "default_tags": default_tag_names(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        use colored::Colorize;
        println!("{}", "Configured tags:".bold());
        for tag in &config.tags {
            if global.no_color {
                println!("  - {}", tag);
            } else {
                let color = priority_to_color(TodoPriority::from_tag(tag));
                println!("  - {}", tag.color(color));
            }
        }
    }

    Ok(())
}
