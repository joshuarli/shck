/// Supported shell dialects for parser syntax decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ShellDialect {
    /// POSIX-style parsing used for `sh`, `dash`, and generic portable shell input.
    Posix,
    /// mksh-specific parsing.
    Mksh,
    /// Bash parsing.
    #[default]
    Bash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DialectFeatures {
    pub(super) double_bracket: bool,
    pub(super) arithmetic_command: bool,
    pub(super) arithmetic_for: bool,
    pub(super) function_keyword: bool,
    pub(super) select_loop: bool,
    pub(super) coproc_keyword: bool,
}

impl ShellDialect {
    /// Infer a parser dialect from a command name, shebang interpreter name,
    /// or user-facing shell selector.
    ///
    /// Unknown names fall back to [`ShellDialect::Bash`], matching Shuck's
    /// default parsing mode.
    pub fn from_name(name: &str) -> Self {
        match name.trim().to_ascii_lowercase().as_str() {
            "sh" | "dash" | "ksh" | "posix" => Self::Posix,
            "mksh" => Self::Mksh,
            _ => Self::Bash,
        }
    }

    pub(super) const fn features(self) -> DialectFeatures {
        match self {
            Self::Posix => DialectFeatures {
                double_bracket: false,
                arithmetic_command: false,
                arithmetic_for: false,
                function_keyword: true,
                select_loop: false,
                coproc_keyword: false,
            },
            Self::Mksh => DialectFeatures {
                double_bracket: true,
                arithmetic_command: true,
                arithmetic_for: false,
                function_keyword: true,
                select_loop: true,
                coproc_keyword: false,
            },
            Self::Bash => DialectFeatures {
                double_bracket: true,
                arithmetic_command: true,
                arithmetic_for: true,
                function_keyword: true,
                select_loop: true,
                coproc_keyword: true,
            },
        }
    }
}

/// Complete shell parsing profile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShellProfile {
    /// Shell dialect to parse.
    pub dialect: ShellDialect,
}

impl ShellProfile {
    /// Build the parser's native profile for `dialect`.
    pub fn native(dialect: ShellDialect) -> Self {
        Self { dialect }
    }
}
