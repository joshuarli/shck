use std::ffi::OsStr;
use std::fmt;
use std::str::FromStr;

use shuck_formatter::IndentStyle;
use shuck_linter::RuleSelector;
use shuck_linter::ShellDialect as LinterShellDialect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatDialectArg {
    Auto,
    Bash,
    Posix,
    Mksh,
}

impl FromStr for FormatDialectArg {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "bash" => Ok(Self::Bash),
            "posix" => Ok(Self::Posix),
            "mksh" => Ok(Self::Mksh),
            _ => Err(format!(
                "invalid dialect '{value}'; expected one of: auto, bash, posix, mksh"
            )),
        }
    }
}

impl From<FormatDialectArg> for shuck_formatter::ShellDialect {
    fn from(value: FormatDialectArg) -> Self {
        match value {
            FormatDialectArg::Auto => Self::Auto,
            FormatDialectArg::Bash => Self::Bash,
            FormatDialectArg::Posix => Self::Posix,
            FormatDialectArg::Mksh => Self::Mksh,
                    }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatIndentStyleArg {
    Tab,
    Space,
}

impl FromStr for FormatIndentStyleArg {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "tab" => Ok(Self::Tab),
            "space" => Ok(Self::Space),
            _ => Err(format!(
                "invalid indent style '{value}'; expected one of: tab, space"
            )),
        }
    }
}

impl From<FormatIndentStyleArg> for IndentStyle {
    fn from(value: FormatIndentStyleArg) -> Self {
        match value {
            FormatIndentStyleArg::Tab => Self::Tab,
            FormatIndentStyleArg::Space => Self::Space,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckOutputFormatArg {
    Concise,
    #[default]
    Full,
    Json,
    JsonLines,
    Junit,
    Grouped,
    Github,
    Gitlab,
    Rdjson,
    Sarif,
}

impl FromStr for CheckOutputFormatArg {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "concise" => Ok(Self::Concise),
            "full" => Ok(Self::Full),
            "json" => Ok(Self::Json),
            "json-lines" | "jsonlines" => Ok(Self::JsonLines),
            "junit" => Ok(Self::Junit),
            "grouped" => Ok(Self::Grouped),
            "github" => Ok(Self::Github),
            "gitlab" => Ok(Self::Gitlab),
            "rdjson" => Ok(Self::Rdjson),
            "sarif" => Ok(Self::Sarif),
            _ => Err(format!(
                "invalid output format '{value}'; expected one of: concise, full, json, json-lines, junit, grouped, github, gitlab, rdjson, sarif"
            )),
        }
    }
}

impl fmt::Display for CheckOutputFormatArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Concise => write!(f, "concise"),
            Self::Full => write!(f, "full"),
            Self::Json => write!(f, "json"),
            Self::JsonLines => write!(f, "json-lines"),
            Self::Junit => write!(f, "junit"),
            Self::Grouped => write!(f, "grouped"),
            Self::Github => write!(f, "github"),
            Self::Gitlab => write!(f, "gitlab"),
            Self::Rdjson => write!(f, "rdjson"),
            Self::Sarif => write!(f, "sarif"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalColor {
    Auto,
    Always,
    Never,
}

impl FromStr for TerminalColor {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(format!(
                "invalid color choice '{value}'; expected one of: auto, always, never"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedShellArg {
    Bash,
    Gbash,
    Bashkit,
    Dash,
    Mksh,
    Busybox,
}

impl FromStr for ManagedShellArg {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "bash" => Ok(Self::Bash),
            "gbash" => Ok(Self::Gbash),
            "bashkit" => Ok(Self::Bashkit),
            "dash" => Ok(Self::Dash),
            "mksh" => Ok(Self::Mksh),
            "busybox" => Ok(Self::Busybox),
            _ => Err(format!(
                "invalid managed shell '{value}'; expected one of: bash, gbash, bashkit, dash, mksh, busybox"
            )),
        }
    }
}

impl From<ManagedShellArg> for shuck_run::Shell {
    fn from(value: ManagedShellArg) -> Self {
        match value {
            ManagedShellArg::Bash => Self::Bash,
            ManagedShellArg::Gbash => Self::Gbash,
            ManagedShellArg::Bashkit => Self::Bashkit,
            ManagedShellArg::Dash => Self::Dash,
            ManagedShellArg::Mksh => Self::Mksh,
            ManagedShellArg::Busybox => Self::Busybox,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternRuleSelectorPair {
    pub pattern: String,
    pub selector: RuleSelector,
}

impl FromStr for PatternRuleSelectorPair {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (pattern, selector) = value
            .rsplit_once(':')
            .ok_or_else(|| "expected <FilePattern>:<RuleCode>".to_owned())?;
        let pattern = pattern.trim();
        let selector = selector.trim();

        if pattern.is_empty() || selector.is_empty() {
            return Err("expected <FilePattern>:<RuleCode>".to_owned());
        }

        Ok(Self {
            pattern: pattern.to_owned(),
            selector: parse_cli_rule_selector(selector)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternShellPair {
    pub pattern: String,
    pub shell: LinterShellDialect,
}

impl FromStr for PatternShellPair {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (pattern, shell) = value
            .rsplit_once(':')
            .ok_or_else(|| "expected <FilePattern>:<Shell>".to_owned())?;
        let pattern = pattern.trim();
        let shell = shell.trim();

        if pattern.is_empty() || shell.is_empty() {
            return Err("expected <FilePattern>:<Shell>".to_owned());
        }

        let shell = LinterShellDialect::from_name(shell);
        if shell == LinterShellDialect::Unknown {
            return Err(
                "expected shell dialect to be one of sh, bash, dash, ksh, mksh".to_owned(),
            );
        }

        Ok(Self {
            pattern: pattern.to_owned(),
            shell,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameworkRootPair {
    pub framework: String,
    pub path: String,
}

impl FromStr for FrameworkRootPair {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (framework, path) = value
            .split_once('=')
            .ok_or_else(|| "expected <Framework>=<Path>".to_owned())?;
        let framework = framework.trim();
        let path = path.trim();

        if framework.is_empty() || path.is_empty() {
            return Err("expected <Framework>=<Path>".to_owned());
        }

        Ok(Self {
            framework: framework.to_owned(),
            path: path.to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternFrameworkNameTriple {
    pub pattern: String,
    pub framework: String,
    pub name: String,
}

impl FromStr for PatternFrameworkNameTriple {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.rsplitn(3, ':');
        let name = parts
            .next()
            .ok_or_else(|| "expected <FilePattern>:<Framework>:<Name>".to_owned())?;
        let framework = parts
            .next()
            .ok_or_else(|| "expected <FilePattern>:<Framework>:<Name>".to_owned())?;
        let pattern = parts
            .next()
            .ok_or_else(|| "expected <FilePattern>:<Framework>:<Name>".to_owned())?;
        let pattern = pattern.trim();
        let framework = framework.trim();
        let name = name.trim();

        if pattern.is_empty() || framework.is_empty() || name.is_empty() {
            return Err("expected <FilePattern>:<Framework>:<Name>".to_owned());
        }

        Ok(Self {
            pattern: pattern.to_owned(),
            framework: framework.to_owned(),
            name: name.to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternPathPair {
    pub pattern: String,
    pub path: String,
}

impl FromStr for PatternPathPair {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (pattern, path) = split_pattern_path_pair(value)
            .ok_or_else(|| "expected <FilePattern>:<Path>".to_owned())?;
        let pattern = pattern.trim();
        let path = path.trim();

        if pattern.is_empty() || path.is_empty() {
            return Err("expected <FilePattern>:<Path>".to_owned());
        }

        Ok(Self {
            pattern: pattern.to_owned(),
            path: path.to_owned(),
        })
    }
}

pub fn split_pattern_path_pair(value: &str) -> Option<(&str, &str)> {
    let bytes = value.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b':' {
            continue;
        }
        if index == 1
            && bytes.first().is_some_and(|byte| byte.is_ascii_alphabetic())
            && bytes
                .get(2)
                .is_some_and(|byte| *byte == b'/' || *byte == b'\\')
        {
            continue;
        }
        return Some((&value[..index], &value[index + 1..]));
    }
    None
}

pub fn parse_cli_rule_selector(value: &str) -> Result<RuleSelector, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("rule selector cannot be empty".to_owned());
    }

    value.parse::<RuleSelector>().map_err(|err| err.to_string())
}

pub fn parse_comma_separated<T: FromStr<Err = String>>(value: &str) -> Result<Vec<T>, String> {
    let mut results = Vec::new();
    for item in value.split(',') {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            return Err("empty element in comma-separated list".to_owned());
        }
        results.push(trimmed.parse::<T>()?);
    }
    Ok(results)
}

pub fn parse_comma_separated_strings(value: &str) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
    for item in value.split(',') {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            return Err("empty element in comma-separated list".to_owned());
        }
        results.push(trimmed.to_owned());
    }
    Ok(results)
}

#[derive(Debug, Clone, Default)]
pub struct RuleSelectionArgs {
    pub select: Option<Vec<RuleSelector>>,
    pub ignore: Vec<RuleSelector>,
    pub extend_select: Vec<RuleSelector>,
    pub per_file_ignores: Option<Vec<PatternRuleSelectorPair>>,
    pub extend_per_file_ignores: Vec<PatternRuleSelectorPair>,
    pub per_file_shell: Option<Vec<PatternShellPair>>,
    pub extend_per_file_shell: Vec<PatternShellPair>,
    pub fixable: Option<Vec<RuleSelector>>,
    pub unfixable: Vec<RuleSelector>,
    pub extend_fixable: Vec<RuleSelector>,
}


#[derive(Debug, Clone, Default)]
pub struct FileSelectionArgs {
    pub exclude: Vec<String>,
    pub extend_exclude: Vec<String>,
    pub respect_gitignore: bool,
    pub no_respect_gitignore: bool,
    pub force_exclude: bool,
    pub no_force_exclude: bool,
}

impl FileSelectionArgs {
    pub fn respect_gitignore(&self) -> bool {
        resolve_bool_flag(self.respect_gitignore, self.no_respect_gitignore, true)
    }

    pub fn force_exclude(&self) -> bool {
        resolve_bool_flag(self.force_exclude, self.no_force_exclude, false)
    }
}

pub fn tri_state_bool(positive: bool, negative: bool) -> Option<bool> {
    match (positive, negative) {
        (false, false) => None,
        (true, false) => Some(true),
        (false, true) => Some(false),
        (true, true) => unreachable!("last flag wins so both cannot be true"),
    }
}

pub fn resolve_bool_flag(positive: bool, negative: bool, default: bool) -> bool {
    match (positive, negative) {
        (false, false) => default,
        (true, false) => true,
        (false, true) => false,
        (true, true) => unreachable!("last flag wins so both cannot be true"),
    }
}

pub fn require_utf8(val: &OsStr, option_name: &str) -> Result<String, crate::cli::error::CliError> {
    match val.to_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(crate::cli::error::CliError::InvalidUtf8 {
            option: option_name.to_string(),
        }),
    }
}
