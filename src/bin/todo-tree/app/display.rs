use colored::Color;
use todo_tree::core::TodoPriority;

pub(crate) fn format_duration(ms: u128) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else {
        format!("{:.2}s", ms as f64 / 1000.0)
    }
}

pub(crate) fn priority_to_color(priority: TodoPriority) -> Color {
    match priority {
        TodoPriority::Critical => Color::Red,
        TodoPriority::High => Color::Yellow,
        TodoPriority::Medium => Color::Cyan,
        TodoPriority::Low => Color::Green,
    }
}
