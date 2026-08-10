//! Path formatting, terminal hyperlink, and tag-coloring helpers.

use super::options::PrintOptions;
use crate::core::Priority;
use colored::{Color, Colorize};
use std::path::Path;

pub(crate) fn format_duration(ms: u128) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else {
        format!("{:.2}s", ms as f64 / 1000.0)
    }
}

pub(crate) fn priority_to_color(priority: Priority) -> Color {
    match priority {
        Priority::Critical => Color::Red,
        Priority::High => Color::Yellow,
        Priority::Medium => Color::Cyan,
        Priority::Low => Color::Green,
    }
}

/// Formats `path` per `options`: absolute if `options.full_paths`,
/// relative to `options.base_path` if set, else as-is.
pub fn format_path(path: &Path, options: &PrintOptions) -> String {
    if options.full_paths {
        path.display().to_string()
    } else if let Some(base) = &options.base_path {
        path.strip_prefix(base)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.display().to_string())
    } else {
        path.display().to_string()
    }
}

/// Builds an OSC 8 terminal hyperlink to `path` at `line`, showing the
/// formatted path as link text. Returns `None` if `options.clickable_links`
/// is off or the terminal doesn't advertise hyperlink support.
pub fn make_clickable_link(path: &Path, line: usize, options: &PrintOptions) -> Option<String> {
    if !options.clickable_links || !supports_hyperlinks() {
        return None;
    }

    let display_path = format_path(path, options);
    let abs_path = path.canonicalize().ok()?;
    let file_url = format!("file://{}:{}", abs_path.display(), line);

    let link = format!(
        "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
        file_url,
        if options.colored {
            display_path.bold().to_string()
        } else {
            display_path
        }
    );

    Some(link)
}

/// Builds an OSC 8 terminal hyperlink to `path` at `line`, showing
/// `"L{line}"` as link text. Returns `None` under the same conditions as
/// [`make_clickable_link`].
pub fn make_line_link(path: &Path, line: usize, options: &PrintOptions) -> Option<String> {
    if !options.clickable_links || !supports_hyperlinks() {
        return None;
    }

    let abs_path = path.canonicalize().ok()?;
    let file_url = format!("file://{}:{}", abs_path.display(), line);
    let display = format!("L{}", line);

    let link = format!(
        "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
        file_url,
        if options.colored {
            display.cyan().to_string()
        } else {
            display
        }
    );

    Some(link)
}

/// Colors `tag` by its derived [`Priority`], or returns it unchanged if
/// `options.colored` is off.
pub fn colorize_tag(tag: &str, options: &PrintOptions) -> String {
    if !options.colored {
        return tag.to_string();
    }

    let color = priority_to_color(Priority::from_tag(tag));
    tag.color(color).bold().to_string()
}

fn supports_hyperlinks() -> bool {
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        let supported_terminals = [
            "iTerm.app",
            "WezTerm",
            "Hyper",
            "Tabby",
            "Alacritty",
            "vscode",
            "VSCodium",
            "Ghostty",
        ];
        if supported_terminals.iter().any(|t| term_program.contains(t)) {
            return true;
        }
    }

    if let Ok(colorterm) = std::env::var("COLORTERM")
        && (colorterm == "truecolor" || colorterm == "24bit")
    {
        return true;
    }

    if std::env::var("VTE_VERSION").is_ok() {
        return true;
    }

    if std::env::var("KONSOLE_VERSION").is_ok() {
        return true;
    }

    false
}

#[cfg(test)]
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
struct EnvGuard {
    term_program: Option<std::ffi::OsString>,
    colorterm: Option<std::ffi::OsString>,
    vte_version: Option<std::ffi::OsString>,
    konsole_version: Option<std::ffi::OsString>,
}

#[cfg(test)]
impl EnvGuard {
    fn clear_all() -> Self {
        let saved = Self {
            term_program: std::env::var_os("TERM_PROGRAM"),
            colorterm: std::env::var_os("COLORTERM"),
            vte_version: std::env::var_os("VTE_VERSION"),
            konsole_version: std::env::var_os("KONSOLE_VERSION"),
        };
        unsafe {
            std::env::remove_var("TERM_PROGRAM");
            std::env::remove_var("COLORTERM");
            std::env::remove_var("VTE_VERSION");
            std::env::remove_var("KONSOLE_VERSION");
        }
        saved
    }
}

#[cfg(test)]
impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.term_program {
                Some(v) => std::env::set_var("TERM_PROGRAM", v),
                None => std::env::remove_var("TERM_PROGRAM"),
            }
            match &self.colorterm {
                Some(v) => std::env::set_var("COLORTERM", v),
                None => std::env::remove_var("COLORTERM"),
            }
            match &self.vte_version {
                Some(v) => std::env::set_var("VTE_VERSION", v),
                None => std::env::remove_var("VTE_VERSION"),
            }
            match &self.konsole_version {
                Some(v) => std::env::set_var("KONSOLE_VERSION", v),
                None => std::env::remove_var("KONSOLE_VERSION"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn options() -> PrintOptions {
        PrintOptions::default()
    }

    #[test]
    fn format_duration_under_a_second_uses_ms() {
        assert_eq!(format_duration(250), "250ms");
    }

    #[test]
    fn format_duration_over_a_second_uses_seconds() {
        assert_eq!(format_duration(1500), "1.50s");
    }

    #[test]
    fn priority_to_color_maps_every_priority() {
        assert_eq!(priority_to_color(Priority::Critical), Color::Red);
        assert_eq!(priority_to_color(Priority::High), Color::Yellow);
        assert_eq!(priority_to_color(Priority::Medium), Color::Cyan);
        assert_eq!(priority_to_color(Priority::Low), Color::Green);
    }

    #[test]
    fn format_path_uses_full_path_when_requested() {
        let opts = PrintOptions {
            full_paths: true,
            ..options()
        };
        let path = PathBuf::from("src/main.rs");
        assert_eq!(format_path(&path, &opts), path.display().to_string());
    }

    #[test]
    fn format_path_strips_base_path_when_set() {
        let opts = PrintOptions {
            base_path: Some(PathBuf::from("/repo")),
            ..options()
        };
        let path = PathBuf::from("/repo/src/main.rs");
        assert_eq!(format_path(&path, &opts), "src/main.rs");
    }

    #[test]
    fn format_path_falls_back_when_strip_prefix_fails() {
        let opts = PrintOptions {
            base_path: Some(PathBuf::from("/other")),
            ..options()
        };
        let path = PathBuf::from("/repo/src/main.rs");
        assert_eq!(format_path(&path, &opts), path.display().to_string());
    }

    #[test]
    fn format_path_uses_display_when_no_base_path() {
        let path = PathBuf::from("src/main.rs");
        assert_eq!(format_path(&path, &options()), path.display().to_string());
    }

    #[test]
    fn colorize_tag_returns_plain_text_when_uncolored() {
        let opts = PrintOptions {
            colored: false,
            ..options()
        };
        assert_eq!(colorize_tag("TODO", &opts), "TODO");
    }

    #[test]
    fn colorize_tag_includes_tag_text_when_colored() {
        let opts = PrintOptions {
            colored: true,
            ..options()
        };
        assert!(colorize_tag("TODO", &opts).contains("TODO"));
    }

    #[test]
    fn make_clickable_link_none_when_disabled() {
        let opts = PrintOptions {
            clickable_links: false,
            ..options()
        };
        assert!(make_clickable_link(Path::new("src/main.rs"), 1, &opts).is_none());
    }

    #[test]
    fn make_line_link_none_when_disabled() {
        let opts = PrintOptions {
            clickable_links: false,
            ..options()
        };
        assert!(make_line_link(Path::new("src/main.rs"), 1, &opts).is_none());
    }

    #[test]
    fn supports_hyperlinks_false_with_no_indicators() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();

        assert!(!supports_hyperlinks());
    }

    #[test]
    fn supports_hyperlinks_true_for_known_term_program() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();
        unsafe {
            std::env::set_var("TERM_PROGRAM", "iTerm.app");
        }

        assert!(supports_hyperlinks());
    }

    #[test]
    fn supports_hyperlinks_true_for_truecolor() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();
        unsafe {
            std::env::set_var("COLORTERM", "truecolor");
        }

        assert!(supports_hyperlinks());
    }

    #[test]
    fn supports_hyperlinks_true_for_vte_version() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();
        unsafe {
            std::env::set_var("VTE_VERSION", "6003");
        }

        assert!(supports_hyperlinks());
    }

    #[test]
    fn supports_hyperlinks_true_for_konsole_version() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();
        unsafe {
            std::env::set_var("KONSOLE_VERSION", "1");
        }

        assert!(supports_hyperlinks());
    }

    #[test]
    fn make_clickable_link_none_when_terminal_unsupported() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();

        let opts = options();
        assert!(make_clickable_link(Path::new("src/main.rs"), 1, &opts).is_none());
    }

    #[test]
    fn make_clickable_link_none_when_path_does_not_exist() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();
        unsafe {
            std::env::set_var("TERM_PROGRAM", "iTerm.app");
        }

        let opts = options();
        let missing = Path::new("/definitely/not/a/real/path/hopefully.rs");
        assert!(make_clickable_link(missing, 1, &opts).is_none());
    }

    #[test]
    fn make_clickable_link_some_for_existing_path_on_supported_terminal() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();
        unsafe {
            std::env::set_var("TERM_PROGRAM", "iTerm.app");
        }

        let opts = PrintOptions {
            colored: true,
            ..options()
        };
        let link = make_clickable_link(Path::new("Cargo.toml"), 1, &opts);
        assert!(link.is_some());
        assert!(link.unwrap().contains("\x1b]8;;file://"));
    }

    #[test]
    fn make_clickable_link_uncolored_variant() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();
        unsafe {
            std::env::set_var("TERM_PROGRAM", "iTerm.app");
        }

        let opts = PrintOptions {
            colored: false,
            ..options()
        };
        let link = make_clickable_link(Path::new("Cargo.toml"), 1, &opts);
        assert!(link.is_some());
    }

    #[test]
    fn make_line_link_some_for_existing_path_on_supported_terminal() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();
        unsafe {
            std::env::set_var("TERM_PROGRAM", "iTerm.app");
        }

        let opts = PrintOptions {
            colored: true,
            ..options()
        };
        let link = make_line_link(Path::new("Cargo.toml"), 42, &opts);
        assert!(link.is_some());
        assert!(link.unwrap().contains("L42"));
    }

    #[test]
    fn make_line_link_uncolored_variant() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();
        unsafe {
            std::env::set_var("TERM_PROGRAM", "iTerm.app");
        }

        let opts = PrintOptions {
            colored: false,
            ..options()
        };
        let link = make_line_link(Path::new("Cargo.toml"), 42, &opts);
        assert!(link.is_some());
    }

    #[test]
    fn make_line_link_none_when_path_does_not_exist() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::clear_all();
        unsafe {
            std::env::set_var("TERM_PROGRAM", "iTerm.app");
        }

        let opts = options();
        let missing = Path::new("/definitely/not/a/real/path/hopefully.rs");
        assert!(make_line_link(missing, 1, &opts).is_none());
    }
}
