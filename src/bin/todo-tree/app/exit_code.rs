//! Maps a top-level command [`Result`] to a process exit code, so scripts
//! can distinguish failure modes without parsing stderr (clap itself already
//! exits with code 2 on argument-parsing errors, before `run()` is called).

use color_eyre::eyre::Result;

/// Success.
pub const OK: i32 = 0;
/// An unclassified application error.
pub const GENERAL: i32 = 1;
/// The `.todorc` config file exists but couldn't be parsed.
pub const CONFIG: i32 = 3;
/// A filesystem operation (resolving a path, reading/writing a file,
/// watching a directory) failed.
pub const IO: i32 = 4;

/// Picks an exit code for `result` by inspecting the error chain's root
/// cause, falling back to [`GENERAL`] for anything unrecognized.
pub fn exit_code_for(result: &Result<()>) -> i32 {
    let Err(report) = result else {
        return OK;
    };

    let mut chain = report.chain();
    if chain.any(|cause| {
        cause.downcast_ref::<toml::de::Error>().is_some()
            || cause.downcast_ref::<serde_json::Error>().is_some()
    }) {
        return CONFIG;
    }

    if report
        .chain()
        .any(|cause| cause.downcast_ref::<std::io::Error>().is_some())
    {
        return IO;
    }

    GENERAL
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::eyre::{Report, eyre};

    #[test]
    fn ok_result_exits_zero() {
        assert_eq!(exit_code_for(&Ok(())), OK);
    }

    #[test]
    fn io_error_exits_with_io_code() {
        let err = std::io::Error::other("boom");
        let result: Result<()> = Err(Report::new(err));
        assert_eq!(exit_code_for(&result), IO);
    }

    #[test]
    fn config_parse_error_exits_with_config_code() {
        let err = toml::from_str::<toml::Value>("not valid { toml").unwrap_err();
        let result: Result<()> = Err(Report::new(err));
        assert_eq!(exit_code_for(&result), CONFIG);
    }

    #[test]
    fn unrecognized_error_exits_with_general_code() {
        let result: Result<()> = Err(eyre!("something went wrong"));
        assert_eq!(exit_code_for(&result), GENERAL);
    }
}
