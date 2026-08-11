//! `tt` binary entry point: a short-name alias for `todo-tree`. Reuses
//! `todo-tree`'s `app` module via `#[path]` so the two binaries share one
//! implementation with no runtime indirection.

#[path = "../todo-tree/app/mod.rs"]
mod app;

fn main() {
    let result = app::run();
    if let Err(report) = &result {
        eprintln!("{report:?}");
    }
    std::process::exit(app::exit_code_for(&result));
}
