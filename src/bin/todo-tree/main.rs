//! `todo-tree` binary entry point.

mod app;

fn main() {
    let result = app::run();
    if let Err(report) = &result {
        eprintln!("{report:?}");
    }
    std::process::exit(app::exit_code_for(&result));
}
