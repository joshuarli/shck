#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionSpec {
    pub short: Option<char>,
    pub long: &'static str,
    pub value_name: Option<&'static str>,
    pub required: bool,
    pub repeatable: bool,
    pub default: Option<&'static str>,
    pub env: Option<&'static str>,
    pub help: &'static str,
    pub heading: Option<&'static str>,
    pub hidden: bool,
}

impl OptionSpec {
    pub const fn new(long: &'static str, help: &'static str) -> Self {
        Self {
            short: None,
            long,
            value_name: None,
            required: false,
            repeatable: false,
            default: None,
            env: None,
            help,
            heading: None,
            hidden: false,
        }
    }

    pub const fn with_short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    pub const fn with_value_name(mut self, value_name: &'static str) -> Self {
        self.value_name = Some(value_name);
        self
    }

    pub const fn with_heading(mut self, heading: &'static str) -> Self {
        self.heading = Some(heading);
        self
    }

    pub const fn with_default(mut self, default: &'static str) -> Self {
        self.default = Some(default);
        self
    }

    pub const fn with_env(mut self, env: &'static str) -> Self {
        self.env = Some(env);
        self
    }

    pub const fn repeatable(mut self) -> Self {
        self.repeatable = true;
        self
    }

    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub const fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn matches_arg(&self, arg: &lexopt::Arg<'_>) -> bool {
        match arg {
            lexopt::Arg::Long(long) => *long == self.long,
            lexopt::Arg::Short(short) => self.short == Some(*short),
            lexopt::Arg::Value(_) => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionalSpec {
    pub name: &'static str,
    pub required: bool,
    pub repeatable: bool,
    pub help: &'static str,
}

impl PositionalSpec {
    pub const fn new(name: &'static str, help: &'static str) -> Self {
        Self {
            name,
            required: false,
            repeatable: false,
            help,
        }
    }

    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub const fn repeatable(mut self) -> Self {
        self.repeatable = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub options: &'static [OptionSpec],
    pub positionals: &'static [PositionalSpec],
    pub subcommands: &'static [CommandSpec],
    pub examples: &'static [&'static str],
}

impl CommandSpec {
    pub const fn new(
        name: &'static str,
        description: &'static str,
        options: &'static [OptionSpec],
        positionals: &'static [PositionalSpec],
        subcommands: &'static [CommandSpec],
        examples: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            description,
            options,
            positionals,
            subcommands,
            examples,
        }
    }
}
