use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;
use lexopt::Arg;
use shuck_config::ConfigArguments;

use crate::ExitStatus;
use crate::cli::command::{CommandSpec, OptionSpec, PositionalSpec};
use crate::cli::error::CliError;
use crate::cli::types::{
    CheckOutputFormatArg, FileSelectionArgs, PatternRuleSelectorPair, PatternShellPair,
    RuleSelectionArgs, parse_cli_rule_selector, parse_comma_separated,
    parse_comma_separated_strings, require_utf8,
};

pub const OPT_FIX: OptionSpec = OptionSpec::new("fix", "Apply safe fixes");
pub const OPT_UNSAFE_FIXES: OptionSpec =
    OptionSpec::new("unsafe-fixes", "Apply unsafe fixes");
pub const OPT_ADD_IGNORE: OptionSpec = OptionSpec::new(
    "add-ignore",
    "Enable automatic additions of shuck ignore directives to failing lines. Optionally provide a reason to append after the codes.",
)
.with_value_name("REASON");
pub const OPT_OUTPUT_FORMAT: OptionSpec = OptionSpec::new(
    "output-format",
    "Output serialization format for violations. The default serialization format is \"full\".",
)
.with_value_name("FORMAT")
.with_env("SHUCK_OUTPUT_FORMAT")
.with_default("full");
pub const OPT_WATCH: OptionSpec =
    OptionSpec::new("watch", "Run in watch mode by re-running whenever files change")
        .with_short('w');
pub const OPT_STDIN_FILENAME: OptionSpec = OptionSpec::new(
    "stdin-filename",
    "The name of the file when passing it through stdin",
)
.with_value_name("STDIN_FILENAME")
.with_heading("Miscellaneous");
pub const OPT_NO_CACHE: OptionSpec =
    OptionSpec::new("no-cache", "Disable cache reads and writes").with_heading("Miscellaneous");
pub const OPT_EXIT_ZERO: OptionSpec = OptionSpec::new(
    "exit-zero",
    "Exit with status code \"0\", even upon detecting lint violations. Parse errors and error-severity diagnostics still fail",
)
.with_short('e')
.with_heading("Miscellaneous");
pub const OPT_EXIT_NON_ZERO_ON_FIX: OptionSpec = OptionSpec::new(
    "exit-non-zero-on-fix",
    "Exit with a non-zero status code if any files were modified via fix, even if no lint violations remain",
)
.with_heading("Miscellaneous");

pub const OPT_SELECT: OptionSpec = OptionSpec::new(
    "select",
    "Comma-separated list of rule selectors to enable (for example `google`, `C`, or `C001`; or ALL to enable all rules)",
)
.with_value_name("RULE_CODE")
.with_heading("Rule selection")
.repeatable();

pub const OPT_IGNORE: OptionSpec = OptionSpec::new(
    "ignore",
    "Comma-separated list of rule selectors to disable",
)
.with_value_name("RULE_CODE")
.with_heading("Rule selection")
.repeatable();

pub const OPT_EXTEND_SELECT: OptionSpec = OptionSpec::new(
    "extend-select",
    "Like --select, but adds additional rule selectors on top of those already specified",
)
.with_value_name("RULE_CODE")
.with_heading("Rule selection")
.repeatable();

pub const OPT_PER_FILE_IGNORES: OptionSpec = OptionSpec::new(
    "per-file-ignores",
    "List of mappings from file pattern to code to exclude",
)
.with_value_name("PER_FILE_IGNORES")
.with_heading("Rule selection")
.repeatable();

pub const OPT_EXTEND_PER_FILE_IGNORES: OptionSpec = OptionSpec::new(
    "extend-per-file-ignores",
    "Like `--per-file-ignores`, but adds additional ignores on top of those already specified",
)
.with_value_name("EXTEND_PER_FILE_IGNORES")
.with_heading("Rule selection")
.repeatable();

pub const OPT_PER_FILE_SHELL: OptionSpec = OptionSpec::new(
    "per-file-shell",
    "List of mappings from file pattern to shell dialect",
)
.with_value_name("PER_FILE_SHELL")
.with_heading("Rule selection")
.repeatable();

pub const OPT_EXTEND_PER_FILE_SHELL: OptionSpec = OptionSpec::new(
    "extend-per-file-shell",
    "Like `--per-file-shell`, but adds additional shell mappings on top of those already specified",
)
.with_value_name("EXTEND_PER_FILE_SHELL")
.with_heading("Rule selection")
.repeatable();

pub const OPT_FIXABLE: OptionSpec = OptionSpec::new(
    "fixable",
    "List of rule selectors to treat as eligible for fix. Only applicable when fix itself is enabled (e.g., via `--fix`)",
)
.with_value_name("RULE_CODE")
.with_heading("Rule selection")
.repeatable();

pub const OPT_UNFIXABLE: OptionSpec = OptionSpec::new(
    "unfixable",
    "List of rule selectors to treat as ineligible for fix. Only applicable when fix itself is enabled (e.g., via `--fix`)",
)
.with_value_name("RULE_CODE")
.with_heading("Rule selection")
.repeatable();

pub const OPT_EXTEND_FIXABLE: OptionSpec = OptionSpec::new(
    "extend-fixable",
    "Like --fixable, but adds additional rule selectors on top of those already specified",
)
.with_value_name("RULE_CODE")
.with_heading("Rule selection")
.repeatable();


pub const OPT_EXCLUDE: OptionSpec = OptionSpec::new(
    "exclude",
    "List of paths, used to omit files and/or directories from analysis",
)
.with_value_name("FILE_PATTERN")
.with_heading("File selection")
.repeatable();

pub const OPT_EXTEND_EXCLUDE: OptionSpec = OptionSpec::new(
    "extend-exclude",
    "Like --exclude, but adds additional files and directories on top of those already excluded",
)
.with_value_name("FILE_PATTERN")
.with_heading("File selection")
.repeatable();

pub const OPT_RESPECT_GITIGNORE: OptionSpec = OptionSpec::new(
    "respect-gitignore",
    "Respect file exclusions via `.gitignore` and other standard ignore files. Use `--no-respect-gitignore` to disable",
)
.with_heading("File selection");

pub const OPT_NO_RESPECT_GITIGNORE: OptionSpec = OptionSpec::new(
    "no-respect-gitignore",
    "Do not respect file exclusions via `.gitignore` and other standard ignore files",
)
.with_heading("File selection")
.hidden();

pub const OPT_FORCE_EXCLUDE: OptionSpec = OptionSpec::new(
    "force-exclude",
    "Enforce exclusions, even for paths passed to shuck directly on the command-line. Use `--no-force-exclude` to disable",
)
.with_heading("File selection");

pub const OPT_NO_FORCE_EXCLUDE: OptionSpec = OptionSpec::new(
    "no-force-exclude",
    "Do not enforce exclusions for paths passed directly on the command-line",
)
.with_heading("File selection")
.hidden();

pub const POS_PATHS: PositionalSpec = PositionalSpec::new(
    "PATHS",
    "Files or directories to check, or `-` to read from stdin",
)
.repeatable();

pub const CHECK_OPTIONS: &[OptionSpec] = &[
    OPT_FIX,
    OPT_UNSAFE_FIXES,
    OPT_ADD_IGNORE,
    OPT_OUTPUT_FORMAT,
    OPT_WATCH,
    OPT_STDIN_FILENAME,
    OPT_NO_CACHE,
    OPT_EXIT_ZERO,
    OPT_EXIT_NON_ZERO_ON_FIX,
    OPT_SELECT,
    OPT_IGNORE,
    OPT_EXTEND_SELECT,
    OPT_PER_FILE_IGNORES,
    OPT_EXTEND_PER_FILE_IGNORES,
    OPT_PER_FILE_SHELL,
    OPT_EXTEND_PER_FILE_SHELL,
    OPT_FIXABLE,
    OPT_UNFIXABLE,
    OPT_EXTEND_FIXABLE,
    OPT_EXCLUDE,
    OPT_EXTEND_EXCLUDE,
    OPT_RESPECT_GITIGNORE,
    OPT_NO_RESPECT_GITIGNORE,
    OPT_FORCE_EXCLUDE,
    OPT_NO_FORCE_EXCLUDE,
];

pub const CHECK_POSITIONALS: &[PositionalSpec] = &[POS_PATHS];

pub const CHECK_SPEC: CommandSpec = CommandSpec::new(
    "check",
    "Lint shell files",
    CHECK_OPTIONS,
    CHECK_POSITIONALS,
    &[],
    &[],
);

#[derive(Debug, Clone)]
pub struct CheckCommand {
    pub fix: bool,
    pub unsafe_fixes: bool,
    pub add_ignore: Option<String>,
    pub output_format: CheckOutputFormatArg,
    pub watch: bool,
    pub paths: Vec<PathBuf>,
    pub stdin_filename: Option<PathBuf>,
    pub rule_selection: RuleSelectionArgs,
    pub file_selection: FileSelectionArgs,
    pub no_cache: bool,
    pub exit_zero: bool,
    pub exit_non_zero_on_fix: bool,
}

impl CheckCommand {
    pub fn respect_gitignore(&self) -> bool {
        self.file_selection.respect_gitignore()
    }

    pub fn force_exclude(&self) -> bool {
        self.file_selection.force_exclude()
    }
}

pub fn parse(parser: &mut lexopt::Parser) -> Result<CheckCommand, CliError> {
    let mut fix = false;
    let mut unsafe_fixes = false;
    let mut add_ignore = None;
    let mut output_format = std::env::var("SHUCK_OUTPUT_FORMAT")
        .ok()
        .and_then(|val| CheckOutputFormatArg::from_str(&val).ok())
        .unwrap_or(CheckOutputFormatArg::Full);
    let mut watch = false;
    let mut paths = Vec::new();
    let mut stdin_filename = None;
    let mut no_cache = false;
    let mut exit_zero = false;
    let mut exit_non_zero_on_fix = false;

    let mut rule_selection = RuleSelectionArgs::default();
    let mut file_selection = FileSelectionArgs::default();

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Long("help") | Arg::Short('h') => {
                let help_text = crate::cli::help::render_command_help("shuck", &CHECK_SPEC, crate::cli::GLOBAL_OPTIONS);
                return Err(CliError::Help(help_text));
            }
            ref a if OPT_FIX.matches_arg(a) => {
                fix = true;
            }
            ref a if OPT_UNSAFE_FIXES.matches_arg(a) => {
                unsafe_fixes = true;
            }
            ref a if OPT_ADD_IGNORE.matches_arg(a) => {
                let val = match parser.optional_value() {
                    Some(os_val) => require_utf8(&os_val, "--add-ignore")?,
                    None => String::new(),
                };
                add_ignore = Some(val);
            }
            ref a if OPT_OUTPUT_FORMAT.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--output-format")?;
                output_format = val_str.parse().map_err(|err: String| CliError::InvalidValue {
                    option: "--output-format".to_string(),
                    value: val_str,
                    reason: err,
                })?;
            }
            ref a if OPT_WATCH.matches_arg(a) => {
                watch = true;
            }
            ref a if OPT_STDIN_FILENAME.matches_arg(a) => {
                let val = parser.value()?;
                stdin_filename = Some(PathBuf::from(val));
            }
            ref a if OPT_NO_CACHE.matches_arg(a) => {
                no_cache = true;
            }
            ref a if OPT_EXIT_ZERO.matches_arg(a) => {
                exit_zero = true;
            }
            ref a if OPT_EXIT_NON_ZERO_ON_FIX.matches_arg(a) => {
                exit_non_zero_on_fix = true;
            }
            ref a if OPT_SELECT.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--select")?;
                let selectors = parse_comma_separated_rules(&val_str, "--select")?;
                rule_selection.select.get_or_insert_with(Vec::new).extend(selectors);
            }
            ref a if OPT_IGNORE.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--ignore")?;
                let selectors = parse_comma_separated_rules(&val_str, "--ignore")?;
                rule_selection.ignore.extend(selectors);
            }
            ref a if OPT_EXTEND_SELECT.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--extend-select")?;
                let selectors = parse_comma_separated_rules(&val_str, "--extend-select")?;
                rule_selection.extend_select.extend(selectors);
            }
            ref a if OPT_PER_FILE_IGNORES.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--per-file-ignores")?;
                let pairs = parse_comma_separated::<PatternRuleSelectorPair>(&val_str)
                    .map_err(|reason| CliError::InvalidValue {
                        option: "--per-file-ignores".to_string(),
                        value: val_str,
                        reason,
                    })?;
                rule_selection.per_file_ignores.get_or_insert_with(Vec::new).extend(pairs);
            }
            ref a if OPT_EXTEND_PER_FILE_IGNORES.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--extend-per-file-ignores")?;
                let pairs = parse_comma_separated::<PatternRuleSelectorPair>(&val_str)
                    .map_err(|reason| CliError::InvalidValue {
                        option: "--extend-per-file-ignores".to_string(),
                        value: val_str,
                        reason,
                    })?;
                rule_selection.extend_per_file_ignores.extend(pairs);
            }
            ref a if OPT_PER_FILE_SHELL.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--per-file-shell")?;
                let pairs = parse_comma_separated::<PatternShellPair>(&val_str)
                    .map_err(|reason| CliError::InvalidValue {
                        option: "--per-file-shell".to_string(),
                        value: val_str,
                        reason,
                    })?;
                rule_selection.per_file_shell.get_or_insert_with(Vec::new).extend(pairs);
            }
            ref a if OPT_EXTEND_PER_FILE_SHELL.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--extend-per-file-shell")?;
                let pairs = parse_comma_separated::<PatternShellPair>(&val_str)
                    .map_err(|reason| CliError::InvalidValue {
                        option: "--extend-per-file-shell".to_string(),
                        value: val_str,
                        reason,
                    })?;
                rule_selection.extend_per_file_shell.extend(pairs);
            }
            ref a if OPT_FIXABLE.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--fixable")?;
                let selectors = parse_comma_separated_rules(&val_str, "--fixable")?;
                rule_selection.fixable.get_or_insert_with(Vec::new).extend(selectors);
            }
            ref a if OPT_UNFIXABLE.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--unfixable")?;
                let selectors = parse_comma_separated_rules(&val_str, "--unfixable")?;
                rule_selection.unfixable.extend(selectors);
            }
            ref a if OPT_EXTEND_FIXABLE.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--extend-fixable")?;
                let selectors = parse_comma_separated_rules(&val_str, "--extend-fixable")?;
                rule_selection.extend_fixable.extend(selectors);
            }
            ref a if OPT_EXCLUDE.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--exclude")?;
                let items = parse_comma_separated_strings(&val_str)
                    .map_err(|reason| CliError::InvalidValue {
                        option: "--exclude".to_string(),
                        value: val_str,
                        reason,
                    })?;
                file_selection.exclude.extend(items);
            }
            ref a if OPT_EXTEND_EXCLUDE.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--extend-exclude")?;
                let items = parse_comma_separated_strings(&val_str)
                    .map_err(|reason| CliError::InvalidValue {
                        option: "--extend-exclude".to_string(),
                        value: val_str,
                        reason,
                    })?;
                file_selection.extend_exclude.extend(items);
            }
            ref a if OPT_RESPECT_GITIGNORE.matches_arg(a) => {
                file_selection.respect_gitignore = true;
                file_selection.no_respect_gitignore = false;
            }
            ref a if OPT_NO_RESPECT_GITIGNORE.matches_arg(a) => {
                file_selection.no_respect_gitignore = true;
                file_selection.respect_gitignore = false;
            }
            ref a if OPT_FORCE_EXCLUDE.matches_arg(a) => {
                file_selection.force_exclude = true;
                file_selection.no_force_exclude = false;
            }
            ref a if OPT_NO_FORCE_EXCLUDE.matches_arg(a) => {
                file_selection.no_force_exclude = true;
                file_selection.force_exclude = false;
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

    if add_ignore.is_some() {
        if fix {
            return Err(CliError::ArgumentConflict {
                message: "the argument '--add-ignore' cannot be used with '--fix'".to_string(),
            });
        }
        if unsafe_fixes {
            return Err(CliError::ArgumentConflict {
                message: "the argument '--add-ignore' cannot be used with '--unsafe-fixes'".to_string(),
            });
        }
        if watch {
            return Err(CliError::ArgumentConflict {
                message: "the argument '--add-ignore' cannot be used with '--watch'".to_string(),
            });
        }
    }

    Ok(CheckCommand {
        fix,
        unsafe_fixes,
        add_ignore,
        output_format,
        watch,
        paths,
        stdin_filename,
        rule_selection,
        file_selection,
        no_cache,
        exit_zero,
        exit_non_zero_on_fix,
    })
}

fn parse_comma_separated_rules(value: &str, option: &str) -> Result<Vec<shuck_linter::RuleSelector>, CliError> {
    let mut results = Vec::new();
    for item in value.split(',') {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            return Err(CliError::InvalidValue {
                option: option.to_string(),
                value: value.to_string(),
                reason: "rule selector cannot be empty".to_string(),
            });
        }
        let selector = parse_cli_rule_selector(trimmed).map_err(|reason| CliError::InvalidValue {
            option: option.to_string(),
            value: value.to_string(),
            reason,
        })?;
        results.push(selector);
    }
    Ok(results)
}

pub fn execute(
    command: CheckCommand,
    config: &ConfigArguments,
    cache_dir: Option<&Path>,
) -> Result<ExitStatus> {
    crate::commands::check::check(command, config, cache_dir)
}

#[cfg(test)]
mod tests {
    use crate::cli::error::CliError;
    use crate::cli::types::CheckOutputFormatArg;

    use super::{CheckCommand, parse};

    fn parse_from<I, T>(args: I) -> Result<CheckCommand, CliError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString>,
    {
        let mut parser = lexopt::Parser::from_iter(args);
        parse(&mut parser)
    }

    #[test]
    fn parses_add_ignore_without_reason() {
        let cmd = parse_from(["check", "--add-ignore"]).unwrap();
        assert_eq!(cmd.add_ignore.as_deref(), Some(""));
    }

    #[test]
    fn parses_add_ignore_with_reason() {
        let cmd = parse_from(["check", "--add-ignore=legacy"]).unwrap();
        assert_eq!(cmd.add_ignore.as_deref(), Some("legacy"));
    }

    #[test]
    fn parses_short_watch_flag() {
        let cmd = parse_from(["check", "-w"]).unwrap();
        assert!(cmd.watch);
    }

    #[test]
    fn parses_long_watch_flag() {
        let cmd = parse_from(["check", "--watch"]).unwrap();
        assert!(cmd.watch);
    }

    #[test]
    fn parses_clusters() {
        let cmd = parse_from(["check", "-we"]).unwrap();
        assert!(cmd.watch);
        assert!(cmd.exit_zero);
    }

    #[test]
    fn parses_all_check_output_formats() {
        for (name, expected) in [
            ("concise", CheckOutputFormatArg::Concise),
            ("full", CheckOutputFormatArg::Full),
            ("json", CheckOutputFormatArg::Json),
            ("json-lines", CheckOutputFormatArg::JsonLines),
            ("junit", CheckOutputFormatArg::Junit),
            ("grouped", CheckOutputFormatArg::Grouped),
            ("github", CheckOutputFormatArg::Github),
            ("gitlab", CheckOutputFormatArg::Gitlab),
            ("rdjson", CheckOutputFormatArg::Rdjson),
            ("sarif", CheckOutputFormatArg::Sarif),
        ] {
            let cmd = parse_from(["check", "--output-format", name]).unwrap();
            assert_eq!(cmd.output_format, expected);
        }
    }

    #[test]
    fn rejects_add_noqa_alias() {
        let err = parse_from(["check", "--add-noqa=legacy"]).unwrap_err();
        assert!(matches!(err, CliError::UnknownOption { .. }));
    }

    #[test]
    fn rejects_add_ignore_with_fix_flags() {
        let err = parse_from(["check", "--add-ignore", "--fix"]).unwrap_err();
        assert!(matches!(err, CliError::ArgumentConflict { .. }));

        let err = parse_from(["check", "--add-ignore", "--unsafe-fixes"]).unwrap_err();
        assert!(matches!(err, CliError::ArgumentConflict { .. }));
    }

    #[test]
    fn rejects_watch_with_add_ignore() {
        let err = parse_from(["check", "-w", "--add-ignore"]).unwrap_err();
        assert!(matches!(err, CliError::ArgumentConflict { .. }));
    }

    #[test]
    fn check_file_selection_negative_flags_override_positive_flags() {
        let cmd = parse_from([
            "check",
            "--respect-gitignore",
            "--no-respect-gitignore",
            "--force-exclude",
            "--no-force-exclude",
        ])
        .unwrap();

        assert!(!cmd.respect_gitignore());
        assert!(!cmd.force_exclude());
    }

    #[test]
    fn zsh_plugin_flags_removed() {
        // zsh support has been removed; plugin resolution flags no longer exist.
    }

    #[test]
    fn rejects_empty_cli_rule_selectors() {
        let err = parse_from(["check", "--select", ""]).unwrap_err();
        assert!(matches!(err, CliError::InvalidValue { .. }));
    }

    #[test]
    fn rejects_empty_cli_rule_selectors_after_value_delimiter() {
        let err = parse_from(["check", "--select", "google,"]).unwrap_err();
        assert!(matches!(err, CliError::InvalidValue { .. }));
    }
}
