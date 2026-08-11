pub mod completions;
pub mod init;
pub mod list;
pub mod man;
pub mod scan;
pub mod stats;
pub mod tags;
pub mod watch;

/// Detects a CI environment via the conventional `CI` env var, set by
/// GitHub Actions, GitLab CI, CircleCI, Travis CI, and most other
/// providers. An explicit `--json`/`--flat` flag always takes precedence
/// over this; it only decides the *default* output format when the user
/// hasn't picked one, so CI logs come out machine-readable by default.
pub(crate) fn is_ci() -> bool {
    match std::env::var("CI") {
        Ok(value) => !value.is_empty() && value != "0" && !value.eq_ignore_ascii_case("false"),
        Err(_) => false,
    }
}
