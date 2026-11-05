use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;
use lexopt::Arg;
use shuck_config::{ConfigArguments, FormatSettingsPatch};

use crate::ExitStatus;
use crate::cli::command::{CommandSpec, OptionSpec, PositionalSpec};
use crate::cli::error::CliError;
pub use crate::cli::types::{
    FileSelectionArgs, FormatDialectArg, FormatIndentStyleArg,
};
use crate::cli::types::{
    parse_comma_separated_strings, require_utf8, tri_state_bool,
};

pub const OPT_CHECK: OptionSpec = OptionSpec::new(
    "check",
    "Avoid writing any formatted files back; instead, exit non-zero if any files would change",
);
pub const OPT_DIFF: OptionSpec = OptionSpec::new(
    "diff",
    "Avoid writing any formatted files back; instead, print a diff for each changed file",
);
pub const OPT_NO_CACHE: OptionSpec =
    OptionSpec::new("no-cache", "Disable cache reads and writes");
pub const OPT_STDIN_FILENAME: OptionSpec = OptionSpec::new(
    "stdin-filename",
    "The name of the file when reading the source from stdin",
)
.with_value_name("STDIN_FILENAME");
pub const OPT_DIALECT: OptionSpec = OptionSpec::new(
    "dialect",
    "Override the auto-discovered shell dialect used for parsing and formatting",
)
.with_value_name("DIALECT");
pub const OPT_INDENT_STYLE: OptionSpec =
    OptionSpec::new("indent-style", "Choose the indentation style")
        .with_value_name("STYLE");
pub const OPT_INDENT_WIDTH: OptionSpec = OptionSpec::new(
    "indent-width",
    "Set the indentation width for space indentation",
)
.with_value_name("WIDTH");
pub const OPT_BINARY_NEXT_LINE: OptionSpec = OptionSpec::new(
    "binary-next-line",
    "Put binary operators on the next line when breaking lists and pipelines",
);
pub const OPT_NO_BINARY_NEXT_LINE: OptionSpec = OptionSpec::new(
    "no-binary-next-line",
    "Do not put binary operators on the next line",
)
.hidden();
pub const OPT_SWITCH_CASE_INDENT: OptionSpec =
    OptionSpec::new("switch-case-indent", "Indent the bodies of `case` branches");
pub const OPT_NO_SWITCH_CASE_INDENT: OptionSpec = OptionSpec::new(
    "no-switch-case-indent",
    "Do not indent the bodies of `case` branches",
)
.hidden();
pub const OPT_SPACE_REDIRECTS: OptionSpec = OptionSpec::new(
    "space-redirects",
    "Insert spaces around redirection operators and targets",
);
pub const OPT_NO_SPACE_REDIRECTS: OptionSpec = OptionSpec::new(
    "no-space-redirects",
    "Do not insert spaces around redirection operators and targets",
)
.hidden();
pub const OPT_KEEP_PADDING: OptionSpec = OptionSpec::new(
    "keep-padding",
    "Preserve source padding when it is safe to do so",
);
pub const OPT_NO_KEEP_PADDING: OptionSpec =
    OptionSpec::new("no-keep-padding", "Do not preserve source padding").hidden();
pub const OPT_FUNCTION_NEXT_LINE: OptionSpec =
    OptionSpec::new("function-next-line", "Put function opening braces on the next line");
pub const OPT_NO_FUNCTION_NEXT_LINE: OptionSpec = OptionSpec::new(
    "no-function-next-line",
    "Do not put function opening braces on the next line",
)
.hidden();
pub const OPT_NEVER_SPLIT: OptionSpec = OptionSpec::new(
    "never-split",
    "Prefer compact layouts and avoid optional splitting",
);
pub const OPT_NO_NEVER_SPLIT: OptionSpec =
    OptionSpec::new("no-never-split", "Do not prefer compact layouts").hidden();
pub const OPT_SIMPLIFY: OptionSpec =
    OptionSpec::new("simplify", "Apply safe simplifications before formatting");
pub const OPT_MINIFY: OptionSpec =
    OptionSpec::new("minify", "Emit a compact minified form and drop comments");

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

pub const POS_FILES: PositionalSpec = PositionalSpec::new(
    "FILES",
    "List of files or directories to format, or `-` to read from stdin",
)
.repeatable();

pub const FORMAT_OPTIONS: &[OptionSpec] = &[
    OPT_CHECK,
    OPT_DIFF,
    OPT_NO_CACHE,
    OPT_STDIN_FILENAME,
    OPT_DIALECT,
    OPT_INDENT_STYLE,
    OPT_INDENT_WIDTH,
    OPT_BINARY_NEXT_LINE,
    OPT_NO_BINARY_NEXT_LINE,
    OPT_SWITCH_CASE_INDENT,
    OPT_NO_SWITCH_CASE_INDENT,
    OPT_SPACE_REDIRECTS,
    OPT_NO_SPACE_REDIRECTS,
    OPT_KEEP_PADDING,
    OPT_NO_KEEP_PADDING,
    OPT_FUNCTION_NEXT_LINE,
    OPT_NO_FUNCTION_NEXT_LINE,
    OPT_NEVER_SPLIT,
    OPT_NO_NEVER_SPLIT,
    OPT_SIMPLIFY,
    OPT_MINIFY,
    OPT_EXCLUDE,
    OPT_EXTEND_EXCLUDE,
    OPT_RESPECT_GITIGNORE,
    OPT_NO_RESPECT_GITIGNORE,
    OPT_FORCE_EXCLUDE,
    OPT_NO_FORCE_EXCLUDE,
];

pub const FORMAT_POSITIONALS: &[PositionalSpec] = &[POS_FILES];

pub const FORMAT_SPEC: CommandSpec = CommandSpec::new(
    "format",
    "Format shell files",
    FORMAT_OPTIONS,
    FORMAT_POSITIONALS,
    &[],
    &[],
);

#[derive(Debug, Clone, Default)]
pub struct FormatCommand {
    pub files: Vec<PathBuf>,
    pub check: bool,
    pub diff: bool,
    pub no_cache: bool,
    pub stdin_filename: Option<PathBuf>,
    pub file_selection: FileSelectionArgs,
    pub dialect: Option<FormatDialectArg>,
    pub indent_style: Option<FormatIndentStyleArg>,
    pub indent_width: Option<u8>,
    pub binary_next_line: bool,
    pub no_binary_next_line: bool,
    pub switch_case_indent: bool,
    pub no_switch_case_indent: bool,
    pub space_redirects: bool,
    pub no_space_redirects: bool,
    pub keep_padding: bool,
    pub no_keep_padding: bool,
    pub function_next_line: bool,
    pub no_function_next_line: bool,
    pub never_split: bool,
    pub no_never_split: bool,
    pub simplify: bool,
    pub minify: bool,
}

impl FormatCommand {
    pub fn format_settings_patch(&self) -> FormatSettingsPatch {
        FormatSettingsPatch {
            dialect: self.dialect.map(Into::into),
            indent_style: self.indent_style.map(Into::into),
            indent_width: self.indent_width,
            binary_next_line: self.binary_next_line(),
            switch_case_indent: self.switch_case_indent(),
            space_redirects: self.space_redirects(),
            keep_padding: self.keep_padding(),
            function_next_line: self.function_next_line(),
            never_split: self.never_split(),
            simplify: self.simplify.then_some(true),
            minify: self.minify.then_some(true),
        }
    }

    pub fn binary_next_line(&self) -> Option<bool> {
        tri_state_bool(self.binary_next_line, self.no_binary_next_line)
    }

    pub fn switch_case_indent(&self) -> Option<bool> {
        tri_state_bool(self.switch_case_indent, self.no_switch_case_indent)
    }

    pub fn space_redirects(&self) -> Option<bool> {
        tri_state_bool(self.space_redirects, self.no_space_redirects)
    }

    pub fn keep_padding(&self) -> Option<bool> {
        tri_state_bool(self.keep_padding, self.no_keep_padding)
    }

    pub fn function_next_line(&self) -> Option<bool> {
        tri_state_bool(self.function_next_line, self.no_function_next_line)
    }

    pub fn never_split(&self) -> Option<bool> {
        tri_state_bool(self.never_split, self.no_never_split)
    }

    pub fn respect_gitignore(&self) -> bool {
        self.file_selection.respect_gitignore()
    }

    pub fn force_exclude(&self) -> bool {
        self.file_selection.force_exclude()
    }
}

pub fn parse(parser: &mut lexopt::Parser) -> Result<FormatCommand, CliError> {
    let mut files = Vec::new();
    let mut check = false;
    let mut diff = false;
    let mut no_cache = false;
    let mut stdin_filename = None;
    let mut file_selection = FileSelectionArgs::default();
    let mut dialect = None;
    let mut indent_style = None;
    let mut indent_width = None;
    let mut binary_next_line = false;
    let mut no_binary_next_line = false;
    let mut switch_case_indent = false;
    let mut no_switch_case_indent = false;
    let mut space_redirects = false;
    let mut no_space_redirects = false;
    let mut keep_padding = false;
    let mut no_keep_padding = false;
    let mut function_next_line = false;
    let mut no_function_next_line = false;
    let mut never_split = false;
    let mut no_never_split = false;
    let mut simplify = false;
    let mut minify = false;

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Long("help") | Arg::Short('h') => {
                let help_text = crate::cli::help::render_command_help("shuck", &FORMAT_SPEC, crate::cli::GLOBAL_OPTIONS);
                return Err(CliError::Help(help_text));
            }
            ref a if OPT_CHECK.matches_arg(a) => {
                check = true;
            }
            ref a if OPT_DIFF.matches_arg(a) => {
                diff = true;
            }
            ref a if OPT_NO_CACHE.matches_arg(a) => {
                no_cache = true;
            }
            ref a if OPT_STDIN_FILENAME.matches_arg(a) => {
                let val = parser.value()?;
                stdin_filename = Some(PathBuf::from(val));
            }
            ref a if OPT_DIALECT.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--dialect")?;
                dialect = Some(val_str.parse().map_err(|err: String| CliError::InvalidValue {
                    option: "--dialect".to_string(),
                    value: val_str,
                    reason: err,
                })?);
            }
            ref a if OPT_INDENT_STYLE.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--indent-style")?;
                indent_style = Some(val_str.parse().map_err(|err: String| CliError::InvalidValue {
                    option: "--indent-style".to_string(),
                    value: val_str,
                    reason: err,
                })?);
            }
            ref a if OPT_INDENT_WIDTH.matches_arg(a) => {
                let val = parser.value()?;
                let val_str = require_utf8(&val, "--indent-width")?;
                let width = u8::from_str(&val_str).map_err(|err| CliError::InvalidValue {
                    option: "--indent-width".to_string(),
                    value: val_str,
                    reason: err.to_string(),
                })?;
                indent_width = Some(width);
            }
            ref a if OPT_BINARY_NEXT_LINE.matches_arg(a) => {
                binary_next_line = true;
                no_binary_next_line = false;
            }
            ref a if OPT_NO_BINARY_NEXT_LINE.matches_arg(a) => {
                no_binary_next_line = true;
                binary_next_line = false;
            }
            ref a if OPT_SWITCH_CASE_INDENT.matches_arg(a) => {
                switch_case_indent = true;
                no_switch_case_indent = false;
            }
            ref a if OPT_NO_SWITCH_CASE_INDENT.matches_arg(a) => {
                no_switch_case_indent = true;
                switch_case_indent = false;
            }
            ref a if OPT_SPACE_REDIRECTS.matches_arg(a) => {
                space_redirects = true;
                no_space_redirects = false;
            }
            ref a if OPT_NO_SPACE_REDIRECTS.matches_arg(a) => {
                no_space_redirects = true;
                space_redirects = false;
            }
            ref a if OPT_KEEP_PADDING.matches_arg(a) => {
                keep_padding = true;
                no_keep_padding = false;
            }
            ref a if OPT_NO_KEEP_PADDING.matches_arg(a) => {
                no_keep_padding = true;
                keep_padding = false;
            }
            ref a if OPT_FUNCTION_NEXT_LINE.matches_arg(a) => {
                function_next_line = true;
                no_function_next_line = false;
            }
            ref a if OPT_NO_FUNCTION_NEXT_LINE.matches_arg(a) => {
                no_function_next_line = true;
                function_next_line = false;
            }
            ref a if OPT_NEVER_SPLIT.matches_arg(a) => {
                never_split = true;
                no_never_split = false;
            }
            ref a if OPT_NO_NEVER_SPLIT.matches_arg(a) => {
                no_never_split = true;
                never_split = false;
            }
            ref a if OPT_SIMPLIFY.matches_arg(a) => {
                simplify = true;
            }
            ref a if OPT_MINIFY.matches_arg(a) => {
                minify = true;
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
                files.push(PathBuf::from(val));
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

    Ok(FormatCommand {
        files,
        check,
        diff,
        no_cache,
        stdin_filename,
        file_selection,
        dialect,
        indent_style,
        indent_width,
        binary_next_line,
        no_binary_next_line,
        switch_case_indent,
        no_switch_case_indent,
        space_redirects,
        no_space_redirects,
        keep_padding,
        no_keep_padding,
        function_next_line,
        no_function_next_line,
        never_split,
        no_never_split,
        simplify,
        minify,
    })
}

pub fn execute(
    command: FormatCommand,
    config: &ConfigArguments,
    cache_dir: Option<&Path>,
) -> Result<ExitStatus> {
    crate::format(command, config, cache_dir)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cli::error::CliError;
    use crate::cli::types::{FormatDialectArg, FormatIndentStyleArg};

    use super::{FormatCommand, parse};

    fn parse_from<I, T>(args: I) -> Result<FormatCommand, CliError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString>,
    {
        let mut parser = lexopt::Parser::from_iter(args);
        parse(&mut parser)
    }

    #[test]
    fn parses_format_flags() {
        let cmd = parse_from(["format", "--check", "--diff", "--no-cache", "script.sh"]).unwrap();
        assert!(cmd.check);
        assert!(cmd.diff);
        assert!(cmd.no_cache);
        assert_eq!(cmd.files, vec![PathBuf::from("script.sh")]);
    }

    #[test]
    fn parses_format_options() {
        let cmd = parse_from([
            "format",
            "--dialect",
            "bash",
            "--indent-style",
            "space",
            "--indent-width",
            "4",
            "--binary-next-line",
            "--simplify",
        ])
        .unwrap();

        assert_eq!(cmd.dialect, Some(FormatDialectArg::Bash));
        assert_eq!(cmd.indent_style, Some(FormatIndentStyleArg::Space));
        assert_eq!(cmd.indent_width, Some(4));
        assert_eq!(cmd.binary_next_line(), Some(true));
        assert!(cmd.simplify);
    }

    #[test]
    fn tri_state_overrides() {
        let cmd = parse_from([
            "format",
            "--binary-next-line",
            "--no-binary-next-line",
        ])
        .unwrap();
        assert_eq!(cmd.binary_next_line(), Some(false));
    }
}
