use shuck_ast::{Span, Word};

use crate::Checker;

pub(crate) fn word_expands_only_static_pattern_safe_literals(
    _checker: &Checker<'_>,
    _word: &Word,
) -> bool {
    false
}

pub(crate) fn span_expands_only_static_pattern_safe_literals(
    _checker: &Checker<'_>,
    _span: Span,
    _is_plain_parameter_reference: bool,
    _is_zsh_presence_test: bool,
) -> bool {
    false
}
