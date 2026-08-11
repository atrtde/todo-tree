pub mod completions;
pub mod init;
pub mod list;
pub mod man;
pub mod scan;
pub mod stats;
pub mod tags;
pub mod watch;

use color_eyre::eyre::Result;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use todo_tree::core::ScanResult;
use todo_tree::scanner::Scanner;

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

/// Whether a progress indicator should be shown for the current invocation:
/// stderr must be a TTY (so it doesn't land in redirected/piped output) and
/// this must not be a CI run (which wants quiet, log-friendly output).
pub(crate) fn show_progress() -> bool {
    !is_ci() && io::stderr().is_terminal()
}

/// Runs `scanner.scan(path)`, printing a "Scanning..." indicator to stderr
/// (cleared before returning) if the scan is still running after a second.
/// A silent multi-second scan on a large tree is otherwise indistinguishable
/// from a hang, which the guide calls out directly: print within 100ms or
/// show progress for anything that can take over a second.
pub(crate) fn scan_with_progress(scanner: &Scanner, path: &Path, enabled: bool) -> Result<ScanResult> {
    if !enabled {
        return scanner.scan(path);
    }

    std::thread::scope(|scope| {
        let handle = scope.spawn(|| scanner.scan(path));
        let start = Instant::now();
        let message = format!("Scanning {}...", path.display());
        let mut shown = false;

        while !handle.is_finished() {
            std::thread::sleep(Duration::from_millis(50));
            if !shown && start.elapsed() >= Duration::from_secs(1) {
                eprint!("{message}\r");
                let _ = io::stderr().flush();
                shown = true;
            }
        }

        if shown {
            eprint!("\r{}\r", " ".repeat(message.len()));
            let _ = io::stderr().flush();
        }

        handle.join().expect("scan thread panicked")
    })
}
