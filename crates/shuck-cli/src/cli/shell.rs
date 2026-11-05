use anyhow::Result;
use lexopt::Arg;
use shuck_config::ConfigArguments;

use crate::ExitStatus;
use crate::cli::command::{CommandSpec, OptionSpec};
use crate::cli::error::CliError;
use crate::cli::types::{ManagedShellArg, require_utf8};

pub const OPT_SHELL: OptionSpec = OptionSpec::new(
    "shell",
    "Shell interpreter name (`bash`, `gbash`, `bashkit`, `zsh`, `dash`, `mksh`, or Linux-only `busybox`)",
)
.with_short('s')
.with_value_name("SHELL");

pub const OPT_SHELL_VERSION: OptionSpec = OptionSpec::new(
    "shell-version",
    "Version constraint (for example `5.2`, `>=5.1,<6`, or `latest`)",
)
.with_short('V')
.with_value_name("CONSTRAINT");

pub const OPT_SYSTEM: OptionSpec = OptionSpec::new(
    "system",
    "Use the system-installed interpreter instead of a managed one",
);

pub const OPT_VERBOSE: OptionSpec = OptionSpec::new(
    "verbose",
    "Show resolution and download progress",
)
.with_short('v');

pub const SHELL_OPTIONS: &[OptionSpec] = &[
    OPT_SHELL,
    OPT_SHELL_VERSION,
    OPT_SYSTEM,
    OPT_VERBOSE,
];

pub const SHELL_SPEC: CommandSpec = CommandSpec::new(
    "shell",
    "Spawn a shell session using a managed interpreter",
    SHELL_OPTIONS,
    &[],
    &[],
    &[],
);

#[derive(Debug, Clone, Default)]
pub struct ShellCommand {
    pub shell: Option<ManagedShellArg>,
    pub shell_version: Option<String>,
    pub system: bool,
    pub verbose: bool,
}

pub fn parse(parser: &mut lexopt::Parser) -> Result<ShellCommand, CliError> {
    let mut shell = None;
    let mut shell_version = None;
    let mut system = false;
    let mut verbose = false;

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Long("help") | Arg::Short('h') => {
                let help_text = crate::cli::help::render_command_help("shuck", &SHELL_SPEC, crate::cli::GLOBAL_OPTIONS);
                return Err(CliError::Help(help_text));
            }
            ref a if OPT_SHELL.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--shell")?;
                shell = Some(val_str.parse().map_err(|err: String| CliError::InvalidValue {
                    option: "--shell".to_string(),
                    value: val_str,
                    reason: err,
                })?);
            }
            ref a if OPT_SHELL_VERSION.matches_arg(a) => {
                let val = parser.value()?;
                shell_version = Some(require_utf8(&val, "--shell-version")?);
            }
            ref a if OPT_SYSTEM.matches_arg(a) => {
                system = true;
            }
            ref a if OPT_VERBOSE.matches_arg(a) => {
                verbose = true;
            }
            Arg::Value(val) => {
                return Err(CliError::UnexpectedArgument {
                    argument: val.to_string_lossy().into_owned(),
                });
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

    Ok(ShellCommand {
        shell,
        shell_version,
        system,
        verbose,
    })
}

pub fn execute(command: ShellCommand, config: &ConfigArguments) -> Result<ExitStatus> {
    crate::commands::runtime::shell(command, config)
}

#[cfg(test)]
mod tests {
    use crate::cli::error::CliError;
    use crate::cli::types::ManagedShellArg;

    use super::{ShellCommand, parse};

    fn parse_from<I, T>(args: I) -> Result<ShellCommand, CliError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString>,
    {
        let mut parser = lexopt::Parser::from_iter(args);
        parse(&mut parser)
    }

    #[test]
    fn parses_shell_flags() {
        let cmd = parse_from([
            "shell",
            "-s",
            "zsh",
            "-V",
            "5.9",
            "--system",
            "-v",
        ])
        .unwrap();

        assert_eq!(cmd.shell, Some(ManagedShellArg::Zsh));
        assert_eq!(cmd.shell_version.as_deref(), Some("5.9"));
        assert!(cmd.system);
        assert!(cmd.verbose);
    }
}
