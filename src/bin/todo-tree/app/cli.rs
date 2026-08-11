use clap::{Args, Parser, Subcommand, ValueHint};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "todo-tree",
    author,
    version,
    about,
    long_about = None,
    before_help = "\
Examples:
  tt                              Scan the current directory
  tt scan ./src --tags TODO,FIXME Scan for specific tags only
  tt watch --json > todos.jsonl   Re-scan on save, streaming JSON
  tt list --filter BUG            List only BUG items, flat
  tt stats --plain                Summary counts, no color
",
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalOptions {
    #[arg(long, global = true, env = "NO_COLOR", help = "Disable colored output")]
    pub no_color: bool,

    #[arg(short, long, global = true, help = "Enable verbose logging")]
    pub verbose: bool,

    #[arg(
        long,
        global = true,
        value_hint = ValueHint::FilePath,
        help = "Path to config file"
    )]
    pub config: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        help = "Never prompt interactively; fail instead of asking for confirmation"
    )]
    pub no_input: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    #[command(visible_alias = "s", about = "Scan files and print TODO matches")]
    Scan(ScanArgs),
    #[command(visible_alias = "w", about = "Watch for file changes and re-scan")]
    Watch(WatchArgs),
    #[command(visible_alias = "l", visible_alias = "ls", about = "List TODO matches")]
    List(ListArgs),
    #[command(visible_alias = "t", about = "Manage configured TODO tags")]
    Tags(TagsArgs),
    #[command(about = "Create a default configuration file")]
    Init(InitArgs),
    #[command(about = "Show summary stats for TODO matches")]
    Stats(StatsArgs),
    #[command(about = "Generate a shell completion script")]
    Completions(CompletionsArgs),
    #[command(about = "Generate a man page")]
    Man,
}

#[derive(Args, Debug, Clone)]
pub struct CompletionsArgs {
    #[arg(help = "Shell to generate completions for")]
    pub shell: Shell,
}

#[derive(Args, Debug, Clone)]
pub struct ScanArgs {
    #[arg(value_hint = ValueHint::AnyPath, help = "Path to scan (defaults to current directory)")]
    pub path: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Tags to search for (comma-separated)"
    )]
    pub tags: Option<Vec<String>>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File patterns to include (glob patterns, comma-separated)"
    )]
    pub include: Option<Vec<String>>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File patterns to exclude (glob patterns, comma-separated)"
    )]
    pub exclude: Option<Vec<String>>,
    #[arg(long, help = "Output results in JSON format")]
    pub json: bool,
    #[arg(long, help = "Print flat output without grouping by file")]
    pub flat: bool,
    #[arg(
        long,
        help = "Plain output: no color, no symbols, no hyperlinks (implies --flat unless --json is set)"
    )]
    pub plain: bool,
    #[arg(
        short,
        long,
        default_value = "0",
        help = "Limit directory traversal depth"
    )]
    pub depth: usize,
    #[arg(long, help = "Follow symlinks when scanning")]
    pub follow_links: bool,
    #[arg(long, help = "Include hidden files and directories")]
    pub hidden: bool,
    #[arg(long, help = "Ignore case when matching tags")]
    pub ignore_case: bool,
    #[arg(long, help = "Allow tags without a trailing colon")]
    pub no_require_colon: bool,
    #[arg(long, default_value = "file", help = "Sort order for results")]
    pub sort: SortOrder,
    #[arg(long, help = "Group output by tag")]
    pub group_by_tag: bool,
}

impl Default for ScanArgs {
    fn default() -> Self {
        Self {
            path: None,
            tags: None,
            include: None,
            exclude: None,
            json: false,
            flat: false,
            plain: false,
            depth: 0,
            follow_links: false,
            hidden: false,
            ignore_case: false,
            no_require_colon: false,
            sort: SortOrder::File,
            group_by_tag: false,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct WatchArgs {
    #[command(flatten)]
    pub scan: ScanArgs,
    #[arg(
        long,
        default_value = "250",
        help = "Debounce window in milliseconds before re-scanning after a file change"
    )]
    pub debounce_ms: u64,
}

#[derive(Args, Debug, Clone, Default)]
pub struct ListArgs {
    #[arg(value_hint = ValueHint::AnyPath, help = "Path to scan (defaults to current directory)")]
    pub path: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Tags to search for (comma-separated)"
    )]
    pub tags: Option<Vec<String>>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File patterns to include (glob patterns, comma-separated)"
    )]
    pub include: Option<Vec<String>>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File patterns to exclude (glob patterns, comma-separated)"
    )]
    pub exclude: Option<Vec<String>>,
    #[arg(long, help = "Output results in JSON format")]
    pub json: bool,
    #[arg(
        long,
        help = "Plain output: no color, no symbols, no hyperlinks"
    )]
    pub plain: bool,
    #[arg(long, help = "Filter results by a specific tag")]
    pub filter: Option<String>,
    #[arg(long, help = "Ignore case when matching tags")]
    pub ignore_case: bool,
    #[arg(long, help = "Allow tags without a trailing colon")]
    pub no_require_colon: bool,
}

#[derive(Args, Debug, Clone)]
pub struct TagsArgs {
    #[arg(long, help = "Show tags in JSON format")]
    pub json: bool,
    #[arg(long, help = "Add a new tag to the configuration")]
    pub add: Option<String>,
    #[arg(long, help = "Remove a tag from the configuration")]
    pub remove: Option<String>,
    #[arg(long, help = "Reset tags to defaults")]
    pub reset: bool,
}

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    #[arg(
        long,
        default_value = "json",
        help = "Configuration format: json or toml"
    )]
    pub format: ConfigFormat,
    #[arg(short, long, help = "Overwrite the config file if it exists")]
    pub force: bool,
}

#[derive(Args, Debug, Clone)]
pub struct StatsArgs {
    #[arg(value_hint = ValueHint::AnyPath, help = "Path to scan (defaults to current directory)")]
    pub path: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Tags to search for (comma-separated)"
    )]
    pub tags: Option<Vec<String>>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File patterns to include (glob patterns, comma-separated)"
    )]
    pub include: Option<Vec<String>>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File patterns to exclude (glob patterns, comma-separated)"
    )]
    pub exclude: Option<Vec<String>>,
    #[arg(long, help = "Output results in JSON format")]
    pub json: bool,
    #[arg(
        long,
        help = "Plain output: no color, no symbols, no hyperlinks"
    )]
    pub plain: bool,
    #[arg(long, help = "Ignore case when matching tags")]
    pub ignore_case: bool,
    #[arg(long, help = "Allow tags without a trailing colon")]
    pub no_require_colon: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum SortOrder {
    #[value(name = "file", help = "Sort by file path")]
    #[default]
    File,
    #[value(name = "line", help = "Sort by line number")]
    Line,
    #[value(name = "priority", help = "Sort by tag priority")]
    Priority,
}

impl From<SortOrder> for todo_tree::core::SortOrder {
    fn from(order: SortOrder) -> Self {
        match order {
            SortOrder::File => Self::File,
            SortOrder::Line => Self::Line,
            SortOrder::Priority => Self::Priority,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ConfigFormat {
    #[default]
    #[value(name = "json", help = "Generate JSON config")]
    Json,
    #[value(name = "toml", help = "Generate TOML config")]
    Toml,
}

impl Cli {
    pub fn get_command(&self) -> Command {
        self.command
            .clone()
            .unwrap_or_else(|| Command::Scan(ScanArgs::default()))
    }
}

impl From<ScanArgs> for ListArgs {
    fn from(scan: ScanArgs) -> Self {
        Self {
            path: scan.path,
            tags: scan.tags,
            include: scan.include,
            exclude: scan.exclude,
            json: scan.json,
            plain: scan.plain,
            filter: None,
            ignore_case: scan.ignore_case,
            no_require_colon: scan.no_require_colon,
        }
    }
}
