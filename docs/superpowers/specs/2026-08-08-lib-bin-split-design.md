# todo-tree: lib/bin split, CI parity, crate metadata

Date: 2026-08-08

## Goal

Reorganize the crate so `todo_tree` is a clean, documented, side-effect-free
library (suitable for docs.rs and reuse), and `todo-tree`/`tt` are thin CLI
binaries built on top of it. Bring CI, `Cargo.toml` metadata, and doc
coverage up to the standard already used in `dotfiles-manager` and
`feedyourai` (both by the same author).

## Non-goals

- No behavior change. Every CLI flag, config format, output format, and
  workflow-template output stays identical.
- No dependency changes beyond what the move requires (none expected).
- `release.yml`, `homebrew.yml`, `todo-tree.yml` workflows are untouched.

## File structure

```
src/
  lib.rs                    #![warn(missing_docs)], re-exports, module docs
  core/
    mod.rs
    types.rs                 TodoItem, ScanResult, ScanSummary, FileResult
    priority.rs               Priority
    tags.rs                    TagDefinition, DEFAULT_TAGS, default_tag_names
  parser.rs                    TodoParser + DEFAULT_REGEX (merged from core/parser.rs)
  scanner.rs                   Scanner, ScanOptions
  config.rs                    Config, CliOptions
  printer/                     unchanged internal layout
    mod.rs / tree.rs / flat.rs / json.rs / summary.rs / options.rs / utils.rs
  utils/
    mod.rs
    display.rs                 format_duration, priority_to_color

src/bin/todo-tree/
  main.rs                     color_eyre::install() + app::run()
  app/
    mod.rs                     Cli::parse() + dispatch (was lib.rs::run)
    cli.rs                      moved from src/cli.rs verbatim
    commands/
      mod.rs                    load_config/save_config/sort_results
      init.rs, list.rs, scan.rs, stats.rs, tags.rs, workflow.rs

src/bin/tt/
  main.rs                     #[path = "../todo-tree/app/mod.rs"] mod app; fn main(){app::run()}
```

`core/parser.rs` today holds only `DEFAULT_REGEX` and collides conceptually
with the top-level `parser.rs` (`TodoParser`). Folding the constant into
`parser.rs` removes that confusion; `core/` becomes pure domain types.

## Crate boundary rule

Library (`src/lib.rs` tree) never prints to stdout/stderr and never reads
CLI args. It exposes: parsing (`TodoParser`), scanning (`Scanner`,
`ScanOptions`), domain types (`core::*`), config file discovery/merge
(`Config`, `CliOptions`), and result formatting (`printer::*`, which writes
to a generic `Write`, not just stdout).

Binaries own `clap` parsing, subcommand dispatch, and all `println!`/exit
code handling. `tt` is a pure alias: it shares `todo-tree`'s `app` module
tree via `#[path]`, no reimplementation.

## Cargo.toml changes

- `[lib]`: drop explicit `path` (defaults to `src/lib.rs`), keep
  `name = "todo_tree"`.
- `[[bin]]` paths updated to `src/bin/todo-tree/main.rs` and
  `src/bin/tt/main.rs`.
- Add `documentation = "https://docs.rs/todo-tree"`.
- Replace `license-file = "LICENSE"` with `license = "GPL-3.0-or-later"`
  (crates.io/docs.rs convention; matches both reference repos).
- `keywords`/`categories` already present — unchanged.
- `include` pattern unchanged (already covers `src/**/*.rs`).

## README

Already uses the raw GitHub asset URL
(`https://raw.githubusercontent.com/alexandretrotel/todo-tree/main/assets/todo-tree.gif`)
for the demo gif. No change needed.

## CI

Replace `.github/workflows/ci.yml` and split a new
`.github/workflows/build-binaries.yml`, matching the structure used in
`dotfiles-manager`/`feedyourai`:

- Triggers: `push: branches: [main]` + `pull_request`.
- `permissions: contents: read, pull-requests: read`.
- No `submodules: true` (no submodules in this repo).
- No `Swatinem/rust-cache` (neither reference repo uses it).
- No Linux system-deps install step (todo-tree has no dbus/X11 deps).
- Jobs in `ci.yml`: `fmt` → `clippy` → `machete` (via
  `bnjbvr/cargo-machete@main`) → `test` (matrix
  `os: [ubuntu-latest, macos-latest, windows-latest]` x
  `rust: [stable, beta]`, excluding beta on macos/windows; plain
  `cargo test --verbose`, no separate release-mode step).
- `build-binaries.yml`: single ubuntu job, `cargo build --release --verbose`,
  then verifies `target/release/todo-tree` and `target/release/tt` exist.

## Documentation coverage

Add `#![warn(missing_docs)]` to `lib.rs`. Every public item in the lib tree
(modules, structs, struct fields, enums, enum variants, functions, consts,
trait impls where applicable) gets a doc comment so the crate builds clean
under the lint and docs.rs output is complete.

## Testing/verification

- `cargo build --workspace` succeeds, both binaries produced.
- `cargo test` passes unchanged (existing parser/scanner unit tests move
  with their files, no test logic changes).
- `cargo clippy --all-targets --all-features -- -D warnings` clean.
- `cargo doc --no-deps` clean under `missing_docs`.
- Manual smoke test: `todo-tree scan`, `tt scan`, `tt tags`, `tt stats`,
  `tt workflow init --force` behave identically to before the move.
