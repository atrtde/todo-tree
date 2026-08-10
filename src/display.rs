//! Terminal display helpers shared by the library's printers and the CLI
//! binaries' own (non-`Printer`) output.

use crate::core::TodoPriority;
use colored::Color;

/// Formats a millisecond duration as `"{ms}ms"` under a second, else
/// `"{s:.2}s"`.
pub fn format_duration(ms: u128) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else {
        format!("{:.2}s", ms as f64 / 1000.0)
    }
}

/// Maps a [`TodoPriority`] to the terminal color used to render it.
pub fn priority_to_color(priority: TodoPriority) -> Color {
    match priority {
        TodoPriority::Critical => Color::Red,
        TodoPriority::High => Color::Yellow,
        TodoPriority::Medium => Color::Cyan,
        TodoPriority::Low => Color::Green,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(priority_to_color(TodoPriority::Critical), Color::Red);
        assert_eq!(priority_to_color(TodoPriority::High), Color::Yellow);
        assert_eq!(priority_to_color(TodoPriority::Medium), Color::Cyan);
        assert_eq!(priority_to_color(TodoPriority::Low), Color::Green);
    }
}
