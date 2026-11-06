use shuck_ast::{static_word_text, Command};
use compact_str::CompactString;

use crate::cfg::{RecordedCommandInfo, RecordedZshCommandEffect};
use crate::source_closure::source_path_template;

pub(super) fn recorded_command_info(
    command: &Command,
    source: &str,
    bash_runtime_vars_enabled: bool,
    _zsh_runtime_vars_enabled: bool,
) -> RecordedCommandInfo {
    match command {
        Command::Simple(command) => {
            recorded_simple_command_info(command, source, bash_runtime_vars_enabled, false)
        }
        _ => RecordedCommandInfo::default(),
    }
}
    #[allow(dead_code)]
pub(super) fn recorded_command_zsh_effects(
    _command: &Command,
    _source: &str,
) -> Vec<RecordedZshCommandEffect> {
    Vec::new()
}

pub(super) fn recorded_simple_command_info(
    command: &shuck_ast::SimpleCommand,
    source: &str,
    bash_runtime_vars_enabled: bool,
    zsh_runtime_vars_enabled: bool,
) -> RecordedCommandInfo {
    let words = std::iter::once(&command.name)
        .chain(command.args.iter())
        .collect::<Vec<_>>();
    let Some(normalized) = shuck_ast::normalize_command_words_owned(words, source) else {
        return RecordedCommandInfo::default();
    };
    recorded_simple_command_info_with(
        command,
        &normalized,
        source,
        bash_runtime_vars_enabled,
        zsh_runtime_vars_enabled,
    )
}

pub(super) fn recorded_simple_command_info_with(
    command: &shuck_ast::SimpleCommand,
    normalized: &shuck_ast::NormalizedCommand<'_>,
    source: &str,
    bash_runtime_vars_enabled: bool,
    _zsh_runtime_vars_enabled: bool,
) -> RecordedCommandInfo {
    let static_callee = normalized.literal_name.as_deref().map(CompactString::from);
    let dynamic_name_span = static_callee
        .is_none()
        .then_some(normalized.body_word_span)
        .flatten();
    let static_args = command
        .args
        .iter()
        .map(|word| static_word_text(word, source).map(CompactString::from))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let resolved = normalized
        .literal_name
        .as_deref()
        .filter(|name| matches!(*name, "source" | "."))
        .and_then(|_| command.args.first())
        .and_then(|word| source_path_template(word, source, bash_runtime_vars_enabled, false));

    RecordedCommandInfo {
        static_callee,
        dynamic_name_span,
        static_args,
        source_path_template: resolved.as_ref().map(|r| r.template.clone()),
        source_path_template_ignored_root: resolved.is_some_and(|r| r.ignored_root),
        zsh_effects: Vec::new(),
    }
}
