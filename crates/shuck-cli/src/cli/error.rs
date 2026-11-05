use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    UnknownCommand {
        name: String,
    },
    UnknownOption {
        option: String,
    },
    MissingValue {
        option: String,
    },
    InvalidValue {
        option: String,
        value: String,
        reason: String,
    },
    MissingRequired {
        name: String,
        usage: String,
    },
    UnexpectedArgument {
        argument: String,
    },
    DuplicateOption {
        option: String,
    },
    ArgumentConflict {
        message: String,
    },
    InvalidUtf8 {
        option: String,
    },
    Help(String),
    Version(String),
    Custom {
        message: String,
        exit_code: u8,
    },
}

impl CliError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Help(_) | Self::Version(_) => 0,
            Self::Custom { exit_code, .. } => *exit_code,
            _ => 2,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCommand { name } => {
                write!(
                    f,
                    "error: unrecognized subcommand '{name}'\n\nUsage: shuck [OPTIONS] <COMMAND>\n\nFor more information, try '--help'."
                )
            }
            Self::UnknownOption { option } => {
                write!(
                    f,
                    "error: unexpected argument '{option}' found\n\nFor more information, try '--help'."
                )
            }
            Self::MissingValue { option } => {
                write!(
                    f,
                    "error: a value is required for '{option}' but none was supplied\n\nFor more information, try '--help'."
                )
            }
            Self::InvalidValue {
                option,
                value,
                reason,
            } => {
                write!(
                    f,
                    "error: invalid value '{value}' for '{option}': {reason}\n\nFor more information, try '--help'."
                )
            }
            Self::MissingRequired { name, usage } => {
                write!(
                    f,
                    "error: the following required arguments were not provided:\n  {name}\n\nUsage: {usage}\n\nFor more information, try '--help'."
                )
            }
            Self::UnexpectedArgument { argument } => {
                write!(
                    f,
                    "error: unexpected argument '{argument}' found\n\nFor more information, try '--help'."
                )
            }
            Self::DuplicateOption { option } => {
                write!(
                    f,
                    "error: the argument '{option}' cannot be used multiple times"
                )
            }
            Self::ArgumentConflict { message } => {
                write!(f, "error: {message}")
            }
            Self::InvalidUtf8 { option } => {
                write!(f, "error: argument '{option}' contains invalid UTF-8 data")
            }
            Self::Help(text) | Self::Version(text) => {
                write!(f, "{text}")
            }
            Self::Custom { message, .. } => {
                write!(f, "{message}")
            }
        }
    }
}

impl std::error::Error for CliError {}

impl From<lexopt::Error> for CliError {
    fn from(err: lexopt::Error) -> Self {
        match err {
            lexopt::Error::MissingValue { option } => Self::MissingValue {
                option: option.unwrap_or_else(|| "option".to_string()),
            },
            lexopt::Error::UnexpectedOption(option) => Self::UnknownOption { option },
            lexopt::Error::UnexpectedValue { value, .. } => Self::UnexpectedArgument {
                argument: value.to_string_lossy().into_owned(),
            },
            lexopt::Error::Custom(box_err) => Self::Custom {
                message: box_err.to_string(),
                exit_code: 2,
            },
            other => Self::Custom {
                message: other.to_string(),
                exit_code: 2,
            },
        }
    }
}

impl From<shuck_config::ConfigCliError> for CliError {
    fn from(err: shuck_config::ConfigCliError) -> Self {
        Self::ArgumentConflict {
            message: err.to_string(),
        }
    }
}
