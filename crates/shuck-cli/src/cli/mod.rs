pub mod check;
pub mod clean;
pub mod command;
pub mod error;
pub mod format;
pub mod help;
pub mod install;
pub mod run;
pub mod shell;
pub mod types;
pub mod version;

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use shuck_config::{ConfigArguments, SingleConfigArgument};

use crate::ExitStatus;
use crate::cli::command::{CommandSpec, OptionSpec};
use crate::cli::error::CliError;
use crate::cli::types::{TerminalColor, require_utf8};

pub const OPT_CONFIG: OptionSpec = OptionSpec::new(
    "config",
    "Either a path to a TOML configuration file (`shuck.toml`), or a TOML `<KEY> = <VALUE>` pair (such as you might find in a `shuck.toml` configuration file) overriding a specific configuration option. Overrides of individual settings using this option always take precedence over all configuration files, including configuration files that were also specified using `--config`",
)
.with_value_name("CONFIG_OPTION")
.with_heading("Global options")
.repeatable();

pub const OPT_ISOLATED: OptionSpec = OptionSpec::new("isolated", "Ignore all configuration files")
    .with_heading("Global options");

pub const OPT_COLOR: OptionSpec = OptionSpec::new("color", "Control when colored output is used")
    .with_value_name("WHEN")
    .with_heading("Global options");

pub const OPT_CACHE_DIR: OptionSpec = OptionSpec::new("cache-dir", "Path to the cache directory")
    .with_value_name("PATH")
    .with_env("SHUCK_CACHE_DIR")
    .with_heading("Miscellaneous");

pub const OPT_HELP: OptionSpec = OptionSpec::new("help", "Print help (see a summary with '-h')")
    .with_short('h');

pub const OPT_VERSION: OptionSpec = OptionSpec::new("version", "Print version")
    .with_short('V');

pub const GLOBAL_OPTIONS: &[OptionSpec] = &[
    OPT_CONFIG,
    OPT_ISOLATED,
    OPT_COLOR,
    OPT_CACHE_DIR,
    OPT_HELP,
    OPT_VERSION,
];

pub const ROOT_SUBCOMMANDS: &[CommandSpec] = &[
    check::CHECK_SPEC,
    run::RUN_SPEC,
    install::INSTALL_SPEC,
    shell::SHELL_SPEC,
    format::FORMAT_SPEC,
    clean::CLEAN_SPEC,
];

pub const ROOT_SPEC: CommandSpec = CommandSpec::new(
    "shuck",
    "Shell checker CLI for shuck",
    GLOBAL_OPTIONS,
    &[],
    ROOT_SUBCOMMANDS,
    &[],
);

#[derive(Debug, Clone)]
pub enum Command {
    Check(Box<check::CheckCommand>),
    Run(run::RunCommand),
    Install(install::InstallCommand),
    Shell(shell::ShellCommand),
    Format(format::FormatCommand),
    Clean(clean::CleanCommand),
}

#[derive(Debug, Clone)]
pub struct Args {
    pub cache_dir: Option<PathBuf>,
    pub(crate) config: ConfigArguments,
    pub(crate) color: Option<TerminalColor>,
    pub command: Command,
}

impl Args {
    pub fn try_parse_from<I, T>(itr: I) -> Result<Self, CliError>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        match parse(itr)? {
            ParsedCli::Run(args) => Ok(*args),
            ParsedCli::Help(text) => Err(CliError::Help(text)),
            ParsedCli::Version(text) => Err(CliError::Version(text)),
        }
    }
}

#[derive(Debug)]
pub enum ParsedCli {
    Run(Box<Args>),
    Help(String),
    Version(String),
}

pub fn parse<I, T>(args: I) -> Result<ParsedCli, CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let all_args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let mut args_iter = all_args.into_iter();

    if let Some(first) = args_iter.next() {
        let first_str = first.to_string_lossy();
        let is_subcommand = matches!(
            first_str.as_ref(),
            "check" | "run" | "install" | "shell" | "format" | "clean"
        );
        let is_option = first_str.starts_with('-');
        if is_subcommand || is_option {
            let mut prepended = vec![first];
            prepended.extend(args_iter);
            args_iter = prepended.into_iter();
        }
    }

    let raw_tokens: Vec<OsString> = args_iter.collect();

    let mut config_options = Vec::new();
    let mut isolated = false;
    let mut color = None;
    let mut cache_dir = std::env::var_os("SHUCK_CACHE_DIR").map(PathBuf::from);

    let mut subcmd_name: Option<String> = None;
    let mut subcmd_tokens: Vec<OsString> = Vec::new();

    let mut idx = 0;
    while idx < raw_tokens.len() {
        let token = &raw_tokens[idx];

        if subcmd_name.is_some() {
            if token == "--config" && idx + 1 < raw_tokens.len() {
                idx += 1;
                let parsed = SingleConfigArgument::parse(&raw_tokens[idx])?;
                config_options.push(parsed);
                idx += 1;
                continue;
            } else if let Some(stripped) = token.to_str().and_then(|s| s.strip_prefix("--config=")) {
                let parsed = SingleConfigArgument::parse(OsStr::new(stripped))?;
                config_options.push(parsed);
                idx += 1;
                continue;
            } else if token == "--isolated" {
                isolated = true;
                idx += 1;
                continue;
            } else if token == "--color" && idx + 1 < raw_tokens.len() {
                idx += 1;
                let val_str = require_utf8(&raw_tokens[idx], "--color")?;
                color = Some(val_str.parse().map_err(|err: String| CliError::InvalidValue {
                    option: "--color".to_string(),
                    value: val_str,
                    reason: err,
                })?);
                idx += 1;
                continue;
            } else if let Some(stripped) = token.to_str().and_then(|s| s.strip_prefix("--color=")) {
                color = Some(stripped.parse().map_err(|err: String| CliError::InvalidValue {
                    option: "--color".to_string(),
                    value: stripped.to_string(),
                    reason: err,
                })?);
                idx += 1;
                continue;
            } else if token == "--cache-dir" && idx + 1 < raw_tokens.len() {
                idx += 1;
                cache_dir = Some(PathBuf::from(&raw_tokens[idx]));
                idx += 1;
                continue;
            } else if let Some(stripped) = token.to_str().and_then(|s| s.strip_prefix("--cache-dir=")) {
                cache_dir = Some(PathBuf::from(stripped));
                idx += 1;
                continue;
            }

            subcmd_tokens.push(token.clone());
            idx += 1;
            continue;
        }

        if token == "-h" || token == "--help" {
            let help_text = help::render_top_level_help("shuck", &ROOT_SPEC);
            return Ok(ParsedCli::Help(help_text));
        }
        if token == "-v" || token == "-V" || token == "--version" {
            return Ok(ParsedCli::Version(version::version_string()));
        }

        if token == "--config" {
            idx += 1;
            if idx >= raw_tokens.len() {
                return Err(CliError::MissingValue {
                    option: "--config".to_string(),
                });
            }
            let parsed = SingleConfigArgument::parse(&raw_tokens[idx])?;
            config_options.push(parsed);
            idx += 1;
            continue;
        }
        if let Some(stripped) = token.to_str().and_then(|s| s.strip_prefix("--config=")) {
            let parsed = SingleConfigArgument::parse(OsStr::new(stripped))?;
            config_options.push(parsed);
            idx += 1;
            continue;
        }
        if token == "--isolated" {
            isolated = true;
            idx += 1;
            continue;
        }
        if token == "--color" {
            idx += 1;
            if idx >= raw_tokens.len() {
                return Err(CliError::MissingValue {
                    option: "--color".to_string(),
                });
            }
            let val_str = require_utf8(&raw_tokens[idx], "--color")?;
            color = Some(val_str.parse().map_err(|err: String| CliError::InvalidValue {
                option: "--color".to_string(),
                value: val_str,
                reason: err,
            })?);
            idx += 1;
            continue;
        }
        if let Some(stripped) = token.to_str().and_then(|s| s.strip_prefix("--color=")) {
            color = Some(stripped.parse().map_err(|err: String| CliError::InvalidValue {
                option: "--color".to_string(),
                value: stripped.to_string(),
                reason: err,
            })?);
            idx += 1;
            continue;
        }
        if token == "--cache-dir" {
            idx += 1;
            if idx >= raw_tokens.len() {
                return Err(CliError::MissingValue {
                    option: "--cache-dir".to_string(),
                });
            }
            cache_dir = Some(PathBuf::from(&raw_tokens[idx]));
            idx += 1;
            continue;
        }
        if let Some(stripped) = token.to_str().and_then(|s| s.strip_prefix("--cache-dir=")) {
            cache_dir = Some(PathBuf::from(stripped));
            idx += 1;
            continue;
        }

        let token_str = require_utf8(token, "COMMAND")?;
        if token_str.starts_with('-') {
            return Err(CliError::UnknownOption {
                option: token_str,
            });
        }

        subcmd_name = Some(token_str);
        idx += 1;
    }

    let Some(name) = subcmd_name else {
        return Err(CliError::MissingRequired {
            name: "<COMMAND>".to_string(),
            usage: "shuck [OPTIONS] <COMMAND>".to_string(),
        });
    };

    macro_rules! handle_subcmd {
        ($expr:expr) => {
            match $expr {
                Ok(val) => val,
                Err(CliError::Help(text)) => return Ok(ParsedCli::Help(text)),
                Err(CliError::Version(text)) => return Ok(ParsedCli::Version(text)),
                Err(err) => return Err(err),
            }
        };
    }

    let command = if name == check::CHECK_SPEC.name {
        let mut parser = lexopt::Parser::from_args(subcmd_tokens);
        let cmd = handle_subcmd!(check::parse(&mut parser));
        Command::Check(Box::new(cmd))
    } else if name == run::RUN_SPEC.name {
        let (prefix, tail) = run::split_script_tail(subcmd_tokens);
        let mut parser = lexopt::Parser::from_args(prefix);
        let mut cmd = handle_subcmd!(run::parse(&mut parser));
        cmd.script_args.extend(tail);
        Command::Run(cmd)
    } else if name == install::INSTALL_SPEC.name {
        let mut parser = lexopt::Parser::from_args(subcmd_tokens);
        let cmd = handle_subcmd!(install::parse(&mut parser));
        Command::Install(cmd)
    } else if name == shell::SHELL_SPEC.name {
        let mut parser = lexopt::Parser::from_args(subcmd_tokens);
        let cmd = handle_subcmd!(shell::parse(&mut parser));
        Command::Shell(cmd)
    } else if name == format::FORMAT_SPEC.name {
        let mut parser = lexopt::Parser::from_args(subcmd_tokens);
        let cmd = handle_subcmd!(format::parse(&mut parser));
        Command::Format(cmd)
    } else if name == clean::CLEAN_SPEC.name {
        let mut parser = lexopt::Parser::from_args(subcmd_tokens);
        let cmd = handle_subcmd!(clean::parse(&mut parser));
        Command::Clean(cmd)
    } else {
        return Err(CliError::UnknownCommand { name });
    };

    let config = ConfigArguments::from_cli(config_options, isolated)?;

    Ok(ParsedCli::Run(Box::new(Args {
        cache_dir,
        config,
        color,
        command,
    })))
}

pub fn execute(args: Args) -> anyhow::Result<ExitStatus> {
    let Args {
        cache_dir,
        config,
        color,
        command,
    } = args;

    if let Some(color_override) = color_override(color, std::env::var_os("FORCE_COLOR")) {
        crate::color::set_override(color_override);
    }

    match command {
        Command::Check(command) => check::execute(*command, &config, cache_dir.as_deref()),
        Command::Run(command) => run::execute(command, &config),
        Command::Install(command) => install::execute(command),
        Command::Shell(command) => shell::execute(command, &config),
        Command::Format(command) => format::execute(command, &config, cache_dir.as_deref()),
        Command::Clean(command) => clean::execute(command, &config, cache_dir.as_deref()),
    }
}

fn color_override(
    color: Option<TerminalColor>,
    env_force_color: Option<OsString>,
) -> Option<bool> {
    match color {
        Some(TerminalColor::Always) => Some(true),
        Some(TerminalColor::Never) => Some(false),
        Some(TerminalColor::Auto) | None => {
            env_force_color.map(|force_color| !force_color.is_empty())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use crate::cli::error::CliError;
    use crate::cli::{Command, ParsedCli, ROOT_SPEC, help, parse, version};

    #[test]
    fn top_level_help_shape_and_ordering() {
        let help_text = help::render_top_level_help("shuck", &ROOT_SPEC);
        assert!(help_text.contains("Shell checker CLI for shuck"));
        assert!(help_text.contains("Usage: shuck [OPTIONS] <COMMAND>"));
        assert!(help_text.contains("Global options:"));
        assert!(help_text.contains("--config <CONFIG_OPTION>"));
        assert!(help_text.contains("--isolated"));
        assert!(help_text.contains("--color <WHEN>"));
        assert!(help_text.contains("Commands:"));
        assert!(help_text.contains("check"));
        assert!(help_text.contains("Format shell files"));
        assert!(help_text.contains("clean"));
        assert!(help_text.contains("run"));
        assert!(help_text.contains("install"));
        assert!(help_text.contains("shell"));
    }

    #[test]
    fn version_and_embedded_sha() {
        let v = version::version_string();
        assert!(v.contains(env!("CARGO_PKG_VERSION")));
        assert!(v.contains("(git "));
        let sha = env!("SHUCK_GIT_SHA");
        assert!(!sha.is_empty());
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn metadata_invariants() {
        assert!(!ROOT_SPEC.description.is_empty());
        let mut cmd_names = std::collections::HashSet::new();

        for sub in ROOT_SPEC.subcommands {
            assert!(!sub.name.is_empty());
            assert!(!sub.description.is_empty());
            assert!(cmd_names.insert(sub.name), "duplicate command name: {}", sub.name);

            let mut long_opts = std::collections::HashSet::new();
            let mut short_opts = std::collections::HashSet::new();

            for opt in sub.options {
                assert!(!opt.long.is_empty());
                assert!(!opt.help.is_empty(), "unexplained option --{} in {}", opt.long, sub.name);
                assert!(long_opts.insert(opt.long), "duplicate long option --{} in {}", opt.long, sub.name);
                if let Some(short) = opt.short {
                    assert!(short_opts.insert(short), "duplicate short option -{} in {}", short, sub.name);
                }
            }

            for pos in sub.positionals {
                assert!(!pos.name.is_empty());
                assert!(!pos.help.is_empty(), "unexplained positional <{}> in {}", pos.name, sub.name);
            }
        }
    }

    #[test]
    fn unknown_and_missing_and_invalid_input() {
        let err = parse(["shuck", "nonexistent"]).unwrap_err();
        assert!(matches!(err, CliError::UnknownCommand { .. }));

        let err = parse(["shuck", "--nonexistent"]).unwrap_err();
        assert!(matches!(err, CliError::UnknownOption { .. }));

        let err = parse(["shuck", "--config"]).unwrap_err();
        assert!(matches!(err, CliError::MissingValue { .. }));

        let err = parse(["shuck", "--color=invalid", "check"]).unwrap_err();
        assert!(matches!(err, CliError::InvalidValue { .. }));
    }

    #[test]
    fn global_config_override_is_available_after_subcommand() {
        let parsed = parse(["shuck", "check", "--config", "format.indent-width = 2"]).unwrap();
        match parsed {
            ParsedCli::Run(args) => {
                assert_eq!(args.config.overrides().format.indent_width, Some(2));
            }
            _ => panic!("expected ParsedCli::Run"),
        }
    }

    #[test]
    fn opaque_tail_preservation() {
        let parsed = parse([
            "shuck", "run", "--shell", "bashkit", "script.sh", "--", "one", "--two",
        ])
        .unwrap();

        match parsed {
            ParsedCli::Run(args) => match args.command {
                Command::Run(run_cmd) => {
                    assert_eq!(run_cmd.script.as_deref(), Some(Path::new("script.sh")));
                    assert_eq!(
                        run_cmd.script_args,
                        vec![OsString::from("one"), OsString::from("--two")]
                    );
                }
                _ => panic!("expected Command::Run"),
            },
            _ => panic!("expected ParsedCli::Run"),
        }
    }

    #[test]
    fn non_utf8_rejection() {
        #[cfg(unix)]
        {
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;

            let invalid_bytes = b"\xff\xfe\xfd";
            let invalid_os_str = OsStr::from_bytes(invalid_bytes);
            let err = parse([OsString::from("shuck"), OsString::from("run"), OsString::from("-s"), invalid_os_str.to_os_string()]).unwrap_err();
            assert!(matches!(err, CliError::InvalidUtf8 { .. }));
        }
    }
}
