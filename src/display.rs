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

/// Case-insensitive Levenshtein (edit) distance between two strings.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.to_lowercase().chars().collect();
    let b: Vec<char> = b.to_lowercase().chars().collect();

    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];

    for (i, &ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (curr[j] + 1)
                .min(prev[j + 1] + 1)
                .min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b.len()]
}

/// Finds the closest string in `candidates` to `input` by edit distance,
/// for "did you mean?" suggestions on likely typos. Returns `None` if
/// nothing is close enough (distance more than a third of the input's
/// length) to be a plausible typo rather than an unrelated value.
pub fn closest_match<'a, I>(input: &str, candidates: I) -> Option<&'a str>
where
    I: IntoIterator<Item = &'a str>,
{
    let max_distance = (input.chars().count() / 2).max(1);

    candidates
        .into_iter()
        .map(|candidate| (candidate, edit_distance(input, candidate)))
        .filter(|(_, distance)| *distance <= max_distance)
        .min_by_key(|(_, distance)| *distance)
        .map(|(candidate, _)| candidate)
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

    #[test]
    fn closest_match_finds_a_single_typo() {
        let candidates = ["TODO", "FIXME", "BUG", "NOTE"];
        assert_eq!(closest_match("TOOD", candidates), Some("TODO"));
        assert_eq!(closest_match("FIXM", candidates), Some("FIXME"));
    }

    #[test]
    fn closest_match_is_case_insensitive() {
        assert_eq!(closest_match("todo", ["TODO", "BUG"]), Some("TODO"));
    }

    #[test]
    fn closest_match_returns_none_when_nothing_close() {
        assert_eq!(closest_match("XYZ", ["TODO", "FIXME", "BUG"]), None);
    }

    #[test]
    fn closest_match_returns_none_for_empty_candidates() {
        assert_eq!(closest_match("TODO", []), None);
    }
}
