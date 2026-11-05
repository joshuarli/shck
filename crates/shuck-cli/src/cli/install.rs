use anyhow::Result;
use lexopt::Arg;

use crate::ExitStatus;
use crate::cli::command::{CommandSpec, OptionSpec, PositionalSpec};
use crate::cli::error::CliError;
use crate::cli::types::{ManagedShellArg, require_utf8};

pub const OPT_LIST: OptionSpec = OptionSpec::new(
    "list",
    "Show available shells and versions instead of installing anything",
);

pub const OPT_REFRESH: OptionSpec = OptionSpec::new(
    "refresh",
    "Force a fresh registry fetch even if the local registry cache is still fresh",
);

pub const POS_SHELL: PositionalSpec = PositionalSpec::new(
    "SHELL",
    "Shell interpreter name (`bash`, `gbash`, `bashkit`, `zsh`, `dash`, `mksh`, or Linux-only `busybox`)",
);

pub const POS_VERSION: PositionalSpec =
    PositionalSpec::new("VERSION", "Version constraint to install");

pub const INSTALL_OPTIONS: &[OptionSpec] = &[OPT_LIST, OPT_REFRESH];

pub const INSTALL_POSITIONALS: &[PositionalSpec] = &[POS_SHELL, POS_VERSION];

pub const INSTALL_SPEC: CommandSpec = CommandSpec::new(
    "install",
    "Pre-install a managed shell interpreter or list available versions",
    INSTALL_OPTIONS,
    INSTALL_POSITIONALS,
    &[],
    &[],
);

#[derive(Debug, Clone, Default)]
pub struct InstallCommand {
    pub list: bool,
    pub refresh: bool,
    pub shell: Option<ManagedShellArg>,
    pub version: Option<String>,
}

pub fn parse(parser: &mut lexopt::Parser) -> Result<InstallCommand, CliError> {
    let mut list = false;
    let mut refresh = false;
    let mut shell = None;
    let mut version = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Long("help") | Arg::Short('h') => {
                let help_text = crate::cli::help::render_command_help("shuck", &INSTALL_SPEC, crate::cli::GLOBAL_OPTIONS);
                return Err(CliError::Help(help_text));
            }
            ref a if OPT_LIST.matches_arg(a) => {
                list = true;
            }
            ref a if OPT_REFRESH.matches_arg(a) => {
                refresh = true;
            }
            Arg::Value(val) => {
                if shell.is_none() {
                    let val_str = require_utf8(&val, "SHELL")?;
                    shell = Some(val_str.parse().map_err(|err: String| CliError::InvalidValue {
                        option: "SHELL".to_string(),
                        value: val_str,
                        reason: err,
                    })?);
                } else if version.is_none() {
                    version = Some(require_utf8(&val, "VERSION")?);
                } else {
                    return Err(CliError::UnexpectedArgument {
                        argument: val.to_string_lossy().into_owned(),
                    });
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

    if !list {
        if shell.is_none() {
            return Err(CliError::MissingRequired {
                name: "<SHELL>".to_string(),
                usage: "shuck install [OPTIONS] <SHELL> <VERSION>".to_string(),
            });
        }
        if version.is_none() {
            return Err(CliError::MissingRequired {
                name: "<VERSION>".to_string(),
                usage: "shuck install [OPTIONS] <SHELL> <VERSION>".to_string(),
            });
        }
    }

    Ok(InstallCommand {
        list,
        refresh,
        shell,
        version,
    })
}

pub fn execute(command: InstallCommand) -> Result<ExitStatus> {
    crate::commands::runtime::install(command)
}

#[cfg(test)]
mod tests {
    use crate::cli::error::CliError;
    use crate::cli::types::ManagedShellArg;

    use super::{InstallCommand, parse};

    fn parse_from<I, T>(args: I) -> Result<InstallCommand, CliError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString>,
    {
        let mut parser = lexopt::Parser::from_iter(args);
        parse(&mut parser)
    }

    #[test]
    fn parses_install_list_without_version() {
        let cmd = parse_from(["install", "--list", "bash"]).unwrap();
        assert!(cmd.list);
        assert_eq!(cmd.shell, Some(ManagedShellArg::Bash));
        assert!(cmd.version.is_none());
    }

    #[test]
    fn parses_install_with_shell_and_version() {
        let cmd = parse_from(["install", "bash", "5.2"]).unwrap();
        assert!(!cmd.list);
        assert_eq!(cmd.shell, Some(ManagedShellArg::Bash));
        assert_eq!(cmd.version.as_deref(), Some("5.2"));
    }

    #[test]
    fn rejects_missing_shell_when_not_list() {
        let err = parse_from(["install"]).unwrap_err();
        assert!(matches!(err, CliError::MissingRequired { .. }));
    }
}
