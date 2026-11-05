use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::Result;
use lexopt::Arg;
use shuck_config::ConfigArguments;

use crate::ExitStatus;
use crate::cli::command::{CommandSpec, OptionSpec, PositionalSpec};
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

pub const OPT_DRY_RUN: OptionSpec = OptionSpec::new(
    "dry-run",
    "Resolve and print the interpreter path without executing",
);

pub const OPT_VERBOSE: OptionSpec = OptionSpec::new(
    "verbose",
    "Show resolution and download progress",
)
.with_short('v');

pub const OPT_COMMAND: OptionSpec = OptionSpec::new(
    "command",
    "Evaluate a command string instead of running a script file",
)
.with_short('c')
.with_value_name("COMMAND");

pub const POS_SCRIPT: PositionalSpec = PositionalSpec::new(
    "SCRIPT",
    "Script path to execute, or `-` to read from stdin",
);

pub const POS_SCRIPT_ARGS: PositionalSpec = PositionalSpec::new(
    "ARGS",
    "Arguments passed through to the script or command",
)
.repeatable();

pub const RUN_OPTIONS: &[OptionSpec] = &[
    OPT_SHELL,
    OPT_SHELL_VERSION,
    OPT_SYSTEM,
    OPT_DRY_RUN,
    OPT_VERBOSE,
    OPT_COMMAND,
];

pub const RUN_POSITIONALS: &[PositionalSpec] = &[POS_SCRIPT, POS_SCRIPT_ARGS];

pub const RUN_SPEC: CommandSpec = CommandSpec::new(
    "run",
    "Run a shell script with a managed interpreter",
    RUN_OPTIONS,
    RUN_POSITIONALS,
    &[],
    &[],
);

#[derive(Debug, Clone, Default)]
pub struct RunCommand {
    pub shell: Option<ManagedShellArg>,
    pub shell_version: Option<String>,
    pub system: bool,
    pub dry_run: bool,
    pub verbose: bool,
    pub command: Option<String>,
    pub script: Option<PathBuf>,
    pub script_args: Vec<OsString>,
}

pub fn split_script_tail<I, T>(args: I) -> (Vec<OsString>, Vec<OsString>)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut prefix = Vec::new();
    let mut tail = Vec::new();
    let mut in_tail = false;

    for arg in args {
        let os_str: OsString = arg.into();
        if in_tail {
            tail.push(os_str);
        } else if os_str == "--" {
            in_tail = true;
        } else {
            prefix.push(os_str);
        }
    }

    (prefix, tail)
}

pub fn parse(parser: &mut lexopt::Parser) -> Result<RunCommand, CliError> {
    let mut shell = None;
    let mut shell_version = None;
    let mut system = false;
    let mut dry_run = false;
    let mut verbose = false;
    let mut command = None;
    let mut script = None;
    let mut script_args = Vec::new();

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Long("help") | Arg::Short('h') => {
                let help_text = crate::cli::help::render_command_help("shuck", &RUN_SPEC, crate::cli::GLOBAL_OPTIONS);
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
            ref a if OPT_DRY_RUN.matches_arg(a) => {
                dry_run = true;
            }
            ref a if OPT_VERBOSE.matches_arg(a) => {
                verbose = true;
            }
            ref a if OPT_COMMAND.matches_arg(a) => {
                let val = parser.value()?;
                command = Some(require_utf8(&val, "--command")?);
            }
            Arg::Value(val) => {
                if script.is_none() {
                    script = Some(PathBuf::from(val));
                } else {
                    script_args.push(val);
                }
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

    if command.is_some() && script.is_some() {
        return Err(CliError::ArgumentConflict {
            message: "the argument '--command' cannot be used with a script path".to_string(),
        });
    }

    Ok(RunCommand {
        shell,
        shell_version,
        system,
        dry_run,
        verbose,
        command,
        script,
        script_args,
    })
}

pub fn execute(command: RunCommand, config: &ConfigArguments) -> Result<ExitStatus> {
    crate::commands::runtime::run(command, config)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use crate::cli::error::CliError;
    use crate::cli::types::ManagedShellArg;

    use super::{RunCommand, parse, split_script_tail};

    fn parse_from<I, T>(args: I) -> Result<RunCommand, CliError>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        let (prefix, tail) = split_script_tail(args);
        let mut parser = lexopt::Parser::from_iter(prefix);
        let mut cmd = parse(&mut parser)?;
        cmd.script_args.extend(tail);
        Ok(cmd)
    }

    #[test]
    fn parses_run_flags_and_passthrough_args() {
        let cmd = parse_from([
            "run",
            "--shell",
            "bash",
            "--shell-version",
            "5.2",
            "--system",
            "--dry-run",
            "--verbose",
            "deploy.sh",
            "--",
            "--env",
            "staging",
        ])
        .unwrap();

        assert_eq!(cmd.shell, Some(ManagedShellArg::Bash));
        assert_eq!(cmd.shell_version.as_deref(), Some("5.2"));
        assert!(cmd.system);
        assert!(cmd.dry_run);
        assert!(cmd.verbose);
        assert_eq!(cmd.script.as_deref(), Some(Path::new("deploy.sh")));
        assert_eq!(
            cmd.script_args,
            vec![OsString::from("--env"), OsString::from("staging")]
        );
    }

    #[test]
    fn parses_command_mode() {
        let cmd = parse_from([
            "run",
            "-s",
            "bash",
            "-c",
            "echo hi",
            "--",
            "one",
            "two",
        ])
        .unwrap();

        assert_eq!(cmd.shell, Some(ManagedShellArg::Bash));
        assert_eq!(cmd.command.as_deref(), Some("echo hi"));
        assert!(cmd.script.is_none());
        assert_eq!(
            cmd.script_args,
            vec![OsString::from("one"), OsString::from("two")]
        );
    }

    #[test]
    fn rejects_conflicting_command_and_script() {
        let err = parse_from(["run", "-c", "echo hi", "script.sh"]).unwrap_err();
        assert!(matches!(err, CliError::ArgumentConflict { .. }));
    }
}
