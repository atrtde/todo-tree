# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## todo-tree-1.0.0

### Breaking Changes
- **Config format**: YAML config support (`.todorc.yaml`/`.todorc.yml`/`config.yaml`/`config.yml`) has been removed in favor of TOML (`.todorc.toml`/`config.toml`). The `yaml_serde` dependency is gone; `toml` has been added. JSON config (`.todorc.json`) is unaffected.
- **`DEFAULT_REGEX` moved**: was `todo_tree::core::DEFAULT_REGEX`, now `todo_tree::parser::DEFAULT_REGEX`.
- **Library/binary split**: CLI argument parsing and subcommand implementations moved out of the `todo_tree` library and into the `todo-tree`/`tt` binaries. The library crate no longer exposes `run()`, `cli`, or `commands` — it is now a documented, side-effect-free library (`todo_tree::{config, core, parser, printer, scanner}`) suitable for use outside the CLI.
- **`todo_tree::display` added**: `priority_to_color` and `format_duration` are now public at `todo_tree::display::{priority_to_color, format_duration}` (previously crate-private `printer` helpers, separately duplicated in the `todo-tree`/`tt` binaries).
- **`todo_tree::core::types` split**: replaced by `todo_tree::core::{todo_item, file_result, summary, scan_result}`. The re-exported types (`TodoItem`, `FileResult`, `ScanSummary`, `ScanResult`) are unchanged at `todo_tree::core::*` and `todo_tree::*`.
- **Global config directory resolution**: now honors `$XDG_CONFIG_HOME` on every platform (previously only the OS-default config directory was checked, and XDG wasn't consulted on macOS/Windows at all).
- **`dirs` replaces `directories-next`** for platform/XDG directory resolution.
- **`Priority` renamed to `TodoPriority`**: was `todo_tree::core::Priority` (`src/core/priority.rs`), now `todo_tree::core::TodoPriority` (`src/core/todo_priority.rs`).
- **`tt workflow` command removed**: `tt workflow init` and everything backing it (GitHub Actions workflow-template generation) is gone. Use the [`todo-tree-action`](https://github.com/alexandretrotel/todo-tree-action) GitHub Action directly in a hand-written workflow file instead.

### Changed
- Reorganized `src/` so the `todo_tree` library and the `todo-tree`/`tt` binaries live in clearly separated trees (`src/` for the library, `src/bin/todo-tree/` and `src/bin/tt/` for the binaries). `tt` shares `todo-tree`'s CLI implementation via `#[path]`, with no runtime indirection.
- Bumped all dependencies to their latest stable versions, pinned at `major.minor` only.
- `Scanner::scan` now walks in parallel (`ignore::WalkBuilder::build_parallel`) across `ScanOptions::threads` workers instead of walking single-threaded; the `threads` option now does what it always claimed to.
- `Scanner` caches its `ignore::Overrides` after the first `scan()` call instead of rebuilding them on every call, since `tt watch` reuses one `Scanner` across many re-scans.
- `TodoParser::parse_file` does a cheap `memchr`-based byte scan for configured tags before validating UTF-8 and running the regex pass, skipping that cost entirely for files that can't match (lockfiles, bundled JS, binaries).
- Clickable OSC 8 hyperlink detection now delegates to the `supports-hyperlinks` crate instead of a hand-rolled `TERM_PROGRAM`/`COLORTERM`/`VTE_VERSION`/`KONSOLE_VERSION` allowlist, picking up terminals (Windows Terminal, kitty, ...) and SSH/TTY handling the old allowlist didn't cover.

### Fixed
- Clickable hyperlinks were previously emitted based on terminal-identifying env vars alone, with no check that stdout was actually a terminal; redirecting or piping `todo-tree`/`tt` output (e.g. `tt scan > out.txt`, `tt scan | grep TODO`) could embed raw OSC 8 escape codes in the non-interactive output. Detection now also requires stdout to be a TTY (or `FORCE_HYPERLINK` to be set).
- `tt stats` always matched tags case-insensitively, hardcoded and ignoring both `.todorc`'s `ignore_case` and (nonexistent) CLI overrides, unlike `scan`/`list`/`watch`, which default to case-sensitive matching. `tt stats` now shares the same config/CLI-driven matching (`--include`, `--exclude`, `--ignore-case`, `--no-require-colon`) and defaults as the other commands.

### Added
- `scan`/`tt` (default), `list`, `watch`, and `stats` now default to JSON output when a `CI` environment variable is set (the convention used by GitHub Actions, GitLab CI, CircleCI, Travis CI, and most other providers), instead of the human-oriented tree/flat/text output used locally. An explicit `--json` or `--flat` flag always takes precedence over this auto-detection.
- `documentation = "https://docs.rs/todo-tree"` in `Cargo.toml`; the crate now builds clean under `#![warn(missing_docs)]` with full public API documentation.
- `tt watch` (alias `tt w`) subcommand: re-scans and reprints on file changes, using `notify` + `notify-debouncer-mini` to coalesce bursts (`--debounce-ms` to tune, default 250ms). File-system events are filtered through the same `.gitignore`/`--include`/`--exclude` rules as a normal scan before triggering a re-scan, so changes under ignored directories (`target/`, `node_modules/`, ...) are skipped.
- `::` back in `DEFAULT_REGEX` as comment marker (removed in 0.3.0 over false positives), now line-start/whitespace-gated so `std::io::Error` still won't match.
- `todo_tree::core::SortOrder` and `ScanResult::sort_by`: result sorting (by file, line, or priority) is now part of the library's public API, not a CLI-only helper.
- `Config::load_or_default` and `Config::save_in_cwd`: the config discovery-with-fallback and save-to-current-directory helpers used by the CLI are now public `Config` methods, usable by library consumers directly.

### CI
- Rewrote `ci.yml` and added a dedicated `build-binaries.yml`, matching the workflow structure used in `dotfiles-manager`/`feedyourai`.
- Simplified `release.yml`: dropped an unnecessary `submodules: true` checkout option (this repo has none), a redundant `-p todo-tree` build flag, and merged the per-OS artifact upload steps into one.
- `todo-tree.yml` (this repo's own PR-scanning workflow) now tracks `todo-tree-action@main`.

## todo-tree-0.6.3

### Changed
- The published crate now uses an explicit `include` allowlist instead of an `exclude` denylist, so only `src/`, `Cargo.toml`, `README.md`, `LICENSE`, and `CHANGELOG.md` are shipped. `.gitignore`, the Nix flake files, and any future non-source files no longer end up in the package.

## todo-tree-0.6.0

### Changed
- Merged the `todo-tree-core` library crate into the `todo-tree` binary crate; the workspace is gone and the project is now a single crate. Core modules live under `todo_tree::core`. No user-facing CLI changes.

### Deprecated
- The standalone `todo-tree-core` crate on crates.io is no longer published. Depend on the `todo-tree` crate and use `todo_tree::core` for the library API instead.

## todo-tree-0.5.1

### Changed
- Removed unused `serde_json` dependency from `todo-tree-core`.
- Removed unused `glob` dependency from `todo-tree`.

### CI
- Added `cargo-machete` job to CI to catch unused dependencies on every PR.

## todo-tree-0.5.0

### Added
- Added `workflow init` command to scaffold `.github/workflows/todo-tree.yml`.
- Generated workflow template now pins `alexandretrotel/todo-tree-action@v1.0.3`.

### Documentation
- Documented GitHub Actions setup with `tt workflow init`.

## todo-tree-0.4.0

### Breaking Changes
- **Core API rename**: `Summary` is now `ScanSummary` and `ScanResult.summary` now uses `ScanSummary`.
- **License change**: Project license changed to GPLv3.

### Added
- **Core parser module** with exported `DEFAULT_REGEX`.
- **CLI display utilities** for color handling.

### Changed
- Refactored CLI scanner, parser, commands, and printer into smaller modules.
- Switched config handling dependencies to `yaml_serde` and `directories-next`.
- Updated CI/release workflows and README metadata/links.

### Removed
- Removed example `.todorc` files from the repository.
- Removed unused test dependencies and test files.

## todo-tree-0.3.0

### Breaking Changes

- **Default scanning now requires uppercase tags with colon**: By default, only `TODO:` format matches, not `todo:` or `TODO ` (without colon). This significantly reduces false positives in real-world codebases.
- **Case-sensitive matching is now the default**: Tags must be uppercase (TODO, FIXME, BUG) to match. Use `--ignore-case` to restore the old behavior.
- **Removed `::` from default comment markers**: Prevents false positives in Rust, C++, and other languages where `::` is used as a scope resolution operator (e.g., `std::io::Error` no longer matches the ERROR tag).

### Added

- **New `--ignore-case` flag**: Ignore case when matching tags (matches TODO, todo, Todo, etc.)
- **New `--no-require-colon` flag**: Allow tags without colon (e.g., `TODO something` without `:`)
- **New `require_colon` and `ignore_case` config options**: Control matching behavior in `.todorc` configuration files
- **Enhanced configuration**: Options can be set in `.todorc.json` or `.todorc.yaml` files
- **Comprehensive test suite**: Added 15+ new tests to prevent false positives

### Fixed

- **False positive**: `std::io::Error` in Rust/C++ code no longer matches ERROR tag
- **False positive**: `std::error` in C++ namespace no longer matches ERROR tag
- **False positive**: Variable names like `ERROR_CODE` no longer match ERROR tag
- **False positive**: Prose like "this is an error" no longer matches ERROR tag
- **False positive**: `Result<T, Error>` in Rust type definitions no longer matches ERROR tag

### Documentation

- Updated README with new scanning behavior and examples
- Added CHANGELOG to track version history
- Added migration guide for users upgrading from 0.2.x
- Improved documentation of default regex pattern

### Tests

- Added test for Rust scope resolution operator (`std::io::Error`)
- Added test for scope resolution with `::` operator
- Added test for C++ namespace resolution  
- Added tests for require-colon behavior
- Added tests for case-sensitive default behavior
- Added tests for variable names containing tag words
- Added tests for mixed-case tag matching

### Migration from 0.2.x to 0.3.0

If you want the old behavior (case-insensitive, no colon required), you have two options:

**Option 1: Command-line flags**
```bash
tt scan --ignore-case --no-require-colon
```

**Option 2: Configuration file** (`.todorc.json`)
```json
{
  "ignore_case": true,
  "require_colon": false
}
```

**Option 3: Configuration file** (`.todorc.yaml`)
```yaml
ignore_case: true
require_colon: false
```

## todo-tree-0.2.1

### Fixed
- Minor bug fixes and performance improvements
- Updated dependencies

### Changed
- Improved error messages
- Better handling of edge cases

## todo-tree-0.2.0

### Added
- Initial public release
- Tree and list output formats
- JSON output support
- Configuration file support (`.todorc.json`, `.todorc.yaml`)
- Configurable tags (TODO, FIXME, BUG, NOTE, HACK, etc.)
- Recursive directory scanning
- `.gitignore` respect
- Priority-based coloring
- Clickable terminal links (OSC 8)
- Multiple comment style support
- Statistics command
- Tags management commands
- Homebrew installation support
- Cargo installation support
- NixOS Flakes support

### Features
- Recursive directory scanning with `.gitignore` support
- Configurable tags with priority levels
- Beautiful tree view output
- Multiple output formats (tree, flat, JSON)
- Configuration file support
- Colored output with priority-based coloring
- Clickable links in supported terminals
- Fast parallel scanning
- Statistics and summary views

---

## Core Library Changelog

### todo-tree-core-0.3.0

#### Breaking Changes

- Changed default tag matching to require uppercase + colon
- Removed `::` from default comment markers to prevent false positives
- Config now uses `ignore_case` instead of `case_sensitive` for clearer semantics

#### Added

- New `require_colon` and `ignore_case` parameters in parser configuration
- Enhanced regex pattern builder with colon requirement option
- New `TodoParser::with_options()` method for full configuration control

#### Deprecated

- `TodoParser::with_regex()` is deprecated in favor of `with_options()`

### todo-tree-core-0.2.1

#### Fixed
- Minor type definition improvements
- Better priority handling

### todo-tree-core-0.2.0

#### Added
- Core types: `TodoItem`, `FileResult`, `ScanResult`, `Summary`
- Priority levels: Critical, High, Medium, Low
- Tag definitions with 17 default tags
- Extensible tag system
- Serialization support with serde
