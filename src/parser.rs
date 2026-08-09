//! Regex-based parsing of TODO-style comments out of file content.

use crate::core::{Priority, TodoItem};
use memchr::{memchr2, memmem};
use regex::{Regex, RegexBuilder};
use std::path::Path;

/// Default regex pattern for matching TODO-style tags in comments.
///
/// This pattern is inspired by the VSCode Todo Tree extension and matches tags
/// that appear after common comment markers.
///
/// Pattern breakdown:
/// - `(//|#|<!--|;|/\*|\*|--)`  - Comment markers for most languages
/// - `\s*`                       - Optional whitespace after comment marker
/// - `($TAGS)`                   - The tag to match (placeholder, replaced at runtime)
/// - `(?:\(([^)]+)\))?`          - Optional author in parentheses
/// - `:`                         - Required colon after tag
/// - `(.*)`                      - The message
///
/// Supported comment syntaxes:
/// ```text
///   //    - C, C++, Java, JavaScript, TypeScript, Rust, Go, Swift, Kotlin
///   #     - Python, Ruby, Shell, YAML, TOML
///   /*    - C-style block comments
///   *     - Block comment continuation lines
///   <!--  - HTML, XML, Markdown comments
///   --    - SQL, Lua, Haskell, Ada
///   ;     - Lisp, Clojure, Assembly, INI files
///   %     - LaTeX, Erlang, MATLAB, Prolog
///   """   - Python docstrings
///   '''   - Python docstrings
///   REM   - Batch files
/// ```
///
/// Note: `::` was removed from default comment markers to prevent false positives
/// in Rust, C++, and other languages where `::` is used as a scope resolution operator
/// (e.g., `std::io::Error`).
pub const DEFAULT_REGEX: &str =
    r#"(//|#|<!--|;|/\*|\*|--|%|"""|'''|REM\s)\s*($TAGS)(?:\(([^)]+)\))?:(.*)"#;

/// Parses TODO-style tags out of file content using a configurable regex.
#[derive(Debug, Clone)]
pub struct TodoParser {
    pattern: Option<Regex>,
    tags: Vec<String>,
    tag_bytes: Vec<Vec<u8>>,
    case_sensitive: bool,
}

impl TodoParser {
    /// Creates a parser for `tags`, requiring an exact-case match and a
    /// trailing colon, using the default comment-marker regex.
    pub fn new(tags: &[String], case_sensitive: bool) -> Self {
        Self::with_options(tags, case_sensitive, true, None)
    }

    /// Creates a parser with full control over case sensitivity, whether a
    /// trailing colon is required, and an optional custom regex (in place
    /// of [`DEFAULT_REGEX`]).
    pub fn with_options(
        tags: &[String],
        case_sensitive: bool,
        require_colon: bool,
        custom_regex: Option<&str>,
    ) -> Self {
        let pattern = Self::build_pattern(tags, case_sensitive, require_colon, custom_regex);
        let tag_bytes = tags.iter().map(|tag| tag.as_bytes().to_vec()).collect();
        Self {
            pattern,
            tags: tags.to_vec(),
            tag_bytes,
            case_sensitive,
        }
    }

    fn build_pattern(
        tags: &[String],
        case_sensitive: bool,
        require_colon: bool,
        custom_regex: Option<&str>,
    ) -> Option<Regex> {
        if tags.is_empty() {
            return None;
        }

        let escaped_tags: Vec<String> = tags.iter().map(|t| regex::escape(t)).collect();
        let tags_alternation = escaped_tags.join("|");

        let mut base_pattern = custom_regex.unwrap_or(DEFAULT_REGEX).to_string();
        if custom_regex.is_none() && !require_colon {
            base_pattern = base_pattern.replace(":(.*)", r"(?:\s*$|(?:(?::|\s+)(.*)))");
        }

        let pattern_string = base_pattern.replace("$TAGS", &tags_alternation);
        let regex = RegexBuilder::new(&pattern_string)
            .case_insensitive(!case_sensitive)
            .multi_line(true)
            .build()
            .expect("Failed to build regex pattern");

        Some(regex)
    }

    /// Parses a single line, returning the matched item if the line
    /// contains one of the parser's tags.
    pub fn parse_line(&self, line: &str, line_number: usize) -> Option<TodoItem> {
        let pattern = self.pattern.as_ref()?;
        if let Some(captures) = pattern.captures(line) {
            let tag_match = captures.get(2)?;
            let author = captures.get(3).map(|m| m.as_str().to_string());
            let message = captures
                .get(4)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            let tag = tag_match.as_str().to_string();
            let column = tag_match.start() + 1;

            let normalized_tag = if self.case_sensitive {
                tag
            } else {
                self.tags
                    .iter()
                    .find(|t| t.eq_ignore_ascii_case(&tag))
                    .cloned()
                    .unwrap_or(tag)
            };

            let priority = Priority::from_tag(&normalized_tag);

            return Some(TodoItem {
                tag: normalized_tag,
                message,
                line: line_number,
                column,
                line_content: Some(line.to_string()),
                author,
                priority,
            });
        }

        None
    }

    /// Parses every line of `content`, returning all matched items in
    /// order.
    pub fn parse_content(&self, content: &str) -> Vec<TodoItem> {
        content
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| self.parse_line(line, idx + 1))
            .collect()
    }

    /// Reads `path` and parses its contents.
    ///
    /// Before paying for UTF-8 validation and a regex pass, this does a
    /// cheap `memchr`-based byte scan for any of the configured tags. Files
    /// that can't possibly match (lockfiles, bundled JS, binaries) are
    /// skipped without ever being validated as UTF-8.
    pub fn parse_file(&self, path: &Path) -> std::io::Result<Vec<TodoItem>> {
        let bytes = std::fs::read(path)?;

        if !contains_any_tag_bytes(&bytes, &self.tag_bytes, self.case_sensitive) {
            return Ok(Vec::new());
        }

        let content = String::from_utf8(bytes)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        Ok(self.parse_content(&content))
    }

    /// The tags this parser was configured with.
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

/// Whether `haystack` contains any of `tags` as a byte substring.
fn contains_any_tag_bytes(haystack: &[u8], tags: &[Vec<u8>], case_sensitive: bool) -> bool {
    tags.iter()
        .any(|tag| contains_tag_bytes(haystack, tag, case_sensitive))
}

/// Whether `haystack` contains `needle` as a byte substring.
///
/// In case-sensitive mode this is a direct [`memmem`] search. In
/// case-insensitive mode, [`memchr2`] finds candidate positions matching
/// either case of `needle`'s first byte (which `memmem` can't do), and each
/// candidate is verified with an ASCII case-insensitive comparison.
fn contains_tag_bytes(haystack: &[u8], needle: &[u8], case_sensitive: bool) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }

    if case_sensitive {
        return memmem::find(haystack, needle).is_some();
    }

    let first = needle[0];
    let (lower, upper) = (first.to_ascii_lowercase(), first.to_ascii_uppercase());

    let mut offset = 0;
    while let Some(pos) = memchr2(lower, upper, &haystack[offset..]) {
        let start = offset + pos;
        let end = start + needle.len();

        if end <= haystack.len() && haystack[start..end].eq_ignore_ascii_case(needle) {
            return true;
        }

        offset = start + 1;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tags() -> Vec<String> {
        vec!["TODO".to_string(), "FIXME".to_string(), "BUG".to_string()]
    }

    fn custom_parser(tags: &[String], case_sensitive: bool) -> TodoParser {
        // Capture layout must match parse_line():
        // 1 = prefix
        // 2 = tag
        // 3 = author
        // 4 = message
        TodoParser::with_options(
            tags,
            case_sensitive,
            true,
            Some(r"(^|\s)($TAGS)(?:\(([^)]+)\))?(?::(.*))?$"),
        )
    }

    #[test]
    fn new_uses_default_options() {
        let parser = TodoParser::new(&tags(), true);
        assert_eq!(parser.tags(), &tags());
        assert!(parser.pattern.is_some());
    }

    #[test]
    fn empty_tags_disable_parsing() {
        let parser = TodoParser::new(&[], true);

        assert!(parser.pattern.is_none());
        assert!(parser.parse_line("// TODO: message", 1).is_none());
        assert!(parser.parse_content("// TODO: message").is_empty());
    }

    #[test]
    fn parse_line_with_custom_regex_extracts_basic_fields() {
        let parser = custom_parser(&tags(), true);
        let item = parser
            .parse_line(" TODO: write more tests", 7)
            .expect("expected TODO item");

        assert_eq!(item.tag, "TODO");
        assert_eq!(item.message, "write more tests");
        assert_eq!(item.author, None);
        assert_eq!(item.line, 7);
        assert_eq!(item.column, 2);
        assert_eq!(
            item.line_content.as_deref(),
            Some(" TODO: write more tests")
        );
        assert_eq!(item.priority, Priority::from_tag("TODO"));
    }

    #[test]
    fn parse_line_with_custom_regex_extracts_author() {
        let parser = custom_parser(&tags(), true);
        let item = parser
            .parse_line(" FIXME(alice): handle edge case", 3)
            .expect("expected FIXME item");

        assert_eq!(item.tag, "FIXME");
        assert_eq!(item.author.as_deref(), Some("alice"));
        assert_eq!(item.message, "handle edge case");
        assert_eq!(item.line, 3);
        assert_eq!(item.column, 2);
        assert_eq!(item.priority, Priority::from_tag("FIXME"));
    }

    #[test]
    fn parse_line_trims_message() {
        let parser = custom_parser(&tags(), true);
        let item = parser
            .parse_line(" TODO:   message with spaces   ", 1)
            .expect("expected TODO item");

        assert_eq!(item.message, "message with spaces");
    }

    #[test]
    fn case_sensitive_parser_rejects_wrong_case() {
        let parser = custom_parser(&tags(), true);

        assert!(parser.parse_line(" todo: lower-case tag", 1).is_none());
        assert!(parser.parse_line(" TODO: upper-case tag", 1).is_some());
    }

    #[test]
    fn case_insensitive_parser_accepts_and_normalizes_tag() {
        let parser = custom_parser(&tags(), false);
        let item = parser
            .parse_line(" todo: lower-case tag", 1)
            .expect("expected TODO item");

        // In case-insensitive mode, the tag should be normalized back
        // to the configured spelling from self.tags.
        assert_eq!(item.tag, "TODO");
        assert_eq!(item.message, "lower-case tag");
        assert_eq!(item.priority, Priority::from_tag("TODO"));
    }

    #[test]
    fn case_insensitive_parser_uses_first_matching_configured_tag_spelling() {
        let tags = vec!["ToDo".to_string(), "FixMe".to_string()];
        let parser = custom_parser(&tags, false);

        let item = parser
            .parse_line(" todo: mixed case normalization", 1)
            .expect("expected ToDo item");

        assert_eq!(item.tag, "ToDo");
        assert_eq!(item.priority, Priority::from_tag("ToDo"));
    }

    #[test]
    fn parse_content_collects_multiple_items_with_correct_line_numbers() {
        let parser = custom_parser(&tags(), false);
        let content = "\
first line
 TODO: first task
nothing here
 fixme(bob): second task
 BUG: third task";

        let items = parser.parse_content(content);

        assert_eq!(items.len(), 3);

        assert_eq!(items[0].tag, "TODO");
        assert_eq!(items[0].message, "first task");
        assert_eq!(items[0].line, 2);

        assert_eq!(items[1].tag, "FIXME");
        assert_eq!(items[1].author.as_deref(), Some("bob"));
        assert_eq!(items[1].message, "second task");
        assert_eq!(items[1].line, 4);

        assert_eq!(items[2].tag, "BUG");
        assert_eq!(items[2].message, "third task");
        assert_eq!(items[2].line, 5);
    }

    #[test]
    fn parse_file_reads_and_parses_content() {
        let parser = custom_parser(&tags(), false);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("todo_parser_test_{unique}.txt"));

        fs::write(
            &path,
            "\
ignore
 TODO: from file
 FIXME(jane): also from file",
        )
        .unwrap();

        let items = parser.parse_file(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(items.len(), 2);

        assert_eq!(items[0].tag, "TODO");
        assert_eq!(items[0].message, "from file");
        assert_eq!(items[0].line, 2);

        assert_eq!(items[1].tag, "FIXME");
        assert_eq!(items[1].author.as_deref(), Some("jane"));
        assert_eq!(items[1].message, "also from file");
        assert_eq!(items[1].line, 3);
    }

    fn temp_file_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("todo_parser_test_{name}_{unique}.bin"))
    }

    #[test]
    fn parse_file_fast_skip_avoids_utf8_error_when_no_tag_present() {
        let parser = TodoParser::new(&tags(), true);
        let path = temp_file_path("no_tag_invalid_utf8");

        // No tag bytes anywhere in this buffer, so the fast-skip should
        // return an empty result without ever validating it as UTF-8.
        fs::write(&path, [0xFF, 0xFE, 0xFD, b'x', b'y', b'z']).unwrap();

        let items = parser.parse_file(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert!(items.is_empty());
    }

    #[test]
    fn parse_file_still_errors_on_invalid_utf8_when_tag_bytes_present() {
        let parser = TodoParser::new(&tags(), true);
        let path = temp_file_path("tag_present_invalid_utf8");

        // "TODO" is present, so the fast-skip can't rule this file out;
        // the invalid UTF-8 must still surface as an error.
        let mut content = b"TODO".to_vec();
        content.extend_from_slice(&[0xFF, 0xFE, 0xFD]);
        fs::write(&path, content).unwrap();

        let result = parser.parse_file(&path);
        let _ = fs::remove_file(&path);

        assert!(result.is_err());
    }

    #[test]
    fn parse_file_fast_skip_finds_case_insensitive_mixed_case_tag() {
        let parser = TodoParser::new(&tags(), false);
        let path = temp_file_path("mixed_case_tag");

        fs::write(&path, "// tOdO: mixed case still matches\n").unwrap();

        let items = parser.parse_file(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, "TODO");
    }

    #[test]
    fn require_colon_true_does_not_match_default_pattern_without_colon() {
        let parser = TodoParser::with_options(&tags(), false, true, None);

        assert!(parser.parse_line("// TODO missing colon", 1).is_none());
        assert!(parser.parse_line("// TODO: has colon", 1).is_some());
    }

    #[test]
    fn require_colon_false_matches_default_pattern_with_or_without_colon() {
        let parser = TodoParser::with_options(&tags(), false, false, None);

        let with_colon = parser.parse_line("// TODO: with colon", 1);
        let with_space = parser.parse_line("// TODO with space", 2);
        let bare_tag = parser.parse_line("// TODO", 3);

        assert!(with_colon.is_some(), "should match with colon");
        assert!(
            with_space.is_some(),
            "should match with space when colon is optional"
        );
        assert!(
            bare_tag.is_some(),
            "should match bare tag when colon is optional"
        );

        let with_space = with_space.unwrap();
        assert_eq!(with_space.tag, "TODO");
        assert_eq!(with_space.message, "with space");

        let bare_tag = bare_tag.unwrap();
        assert_eq!(bare_tag.tag, "TODO");
        assert_eq!(bare_tag.message, "");
    }

    #[test]
    fn require_colon_false_rejects_false_positives() {
        let parser = TodoParser::with_options(&tags(), false, false, None);

        assert!(
            parser.parse_line("// TODO.complete()", 4).is_none(),
            "tag followed by '.' must not match"
        );
        assert!(
            parser.parse_line("// todoList", 5).is_none(),
            "tag embedded in a word must not match"
        );
    }

    #[test]
    fn require_colon_false_documents_double_colon_behavior() {
        let parser = TodoParser::with_options(&tags(), false, false, None);

        let item = parser
            .parse_line("* TODO::module::fn", 6)
            .expect("double-colon form should match current default regex behavior");

        assert_eq!(item.tag, "TODO");
        assert_eq!(item.message, ":module::fn");
    }

    #[test]
    fn custom_regex_can_support_non_default_syntax() {
        let tags = vec!["TODO".to_string(), "FIXME".to_string()];
        let parser = TodoParser::with_options(
            &tags,
            false,
            true,
            // Matches e.g. "[TODO]{alice}: message"
            // 1 = prefix
            // 2 = tag
            // 3 = author
            // 4 = message
            Some(r"(^|\s)\[($TAGS)\](?:\{([^}]+)\})?:(.*)$"),
        );

        let item = parser
            .parse_line("[todo]{alice}: custom format works", 10)
            .expect("expected custom format to match");

        assert_eq!(item.tag, "TODO");
        assert_eq!(item.author.as_deref(), Some("alice"));
        assert_eq!(item.message, "custom format works");
        assert_eq!(item.line, 10);
        assert_eq!(item.priority, Priority::from_tag("TODO"));
    }

    #[test]
    fn default_regex_smoke_test_common_comment_styles() {
        let parser = TodoParser::with_options(&tags(), false, true, None);

        let slash = parser.parse_line("// TODO: implement feature", 1);
        let hash = parser.parse_line("# FIXME: fix the bug", 2);

        assert!(slash.is_some(), "default regex should match // TODO: ...");
        assert!(hash.is_some(), "default regex should match # FIXME: ...");

        let slash = slash.unwrap();
        assert_eq!(slash.tag, "TODO");
        assert_eq!(slash.message, "implement feature");

        let hash = hash.unwrap();
        assert_eq!(hash.tag, "FIXME");
        assert_eq!(hash.message, "fix the bug");
    }

    #[test]
    fn tags_accessor_returns_configured_tags() {
        let tags = vec!["TODO".to_string(), "FIXME".to_string()];
        let parser = TodoParser::new(&tags, true);

        assert_eq!(parser.tags(), &tags);
    }
}
