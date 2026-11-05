use std::path::{Path, PathBuf};

use anyhow::Result;
use lexopt::Arg;
use shuck_config::ConfigArguments;

use crate::ExitStatus;
use crate::cli::command::{CommandSpec, PositionalSpec};
use crate::cli::error::CliError;

pub const POS_PATHS: PositionalSpec = PositionalSpec::new(
    "PATHS",
    "Files or directories whose project caches should be removed",
)
.repeatable();

pub const CLEAN_SPEC: CommandSpec = CommandSpec::new(
    "clean",
    "Remove shuck cache entries for the provided paths' projects",
    &[],
    &[POS_PATHS],
    &[],
    &[],
);

#[derive(Debug, Clone, Default)]
pub struct CleanCommand {
    pub paths: Vec<PathBuf>,
}

pub fn parse(parser: &mut lexopt::Parser) -> Result<CleanCommand, CliError> {
    let mut paths = Vec::new();

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Long("help") | Arg::Short('h') => {
                let help_text = crate::cli::help::render_command_help("shuck", &CLEAN_SPEC, crate::cli::GLOBAL_OPTIONS);
                return Err(CliError::Help(help_text));
            }
            Arg::Value(val) => {
                paths.push(PathBuf::from(val));
            }
            Arg::Long(long) => {
                return Err(CliError::UnknownOption {
                    option: format!("--{long}"),
                });
            }
            Arg::Short(short) => {
                return Err(CliError::UnknownOption {
                    option: format!("-{short}"),
                });
            }
        }
    }

    Ok(CleanCommand { paths })
}

pub fn execute(
    command: CleanCommand,
    config: &ConfigArguments,
    cache_dir: Option<&Path>,
) -> Result<ExitStatus> {
    crate::commands::clean::clean(command, config, cache_dir)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::parse;

    #[test]
    fn parses_paths() {
        let mut parser = lexopt::Parser::from_iter(["clean", "foo.sh", "bar.sh"]);
        let cmd = parse(&mut parser).unwrap();
        assert_eq!(cmd.paths, vec![PathBuf::from("foo.sh"), PathBuf::from("bar.sh")]);
    }

    #[test]
    fn parses_empty_paths() {
        let mut parser = lexopt::Parser::from_iter(["clean"]);
        let cmd = parse(&mut parser).unwrap();
        assert!(cmd.paths.is_empty());
    }
}
