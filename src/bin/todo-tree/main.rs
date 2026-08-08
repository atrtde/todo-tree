//! `todo-tree` binary entry point.

mod app;

fn main() -> color_eyre::eyre::Result<()> {
    app::run()
}
