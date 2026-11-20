use shuck_ast::{
    ArithmeticExpr, BourneParameterExpansion, CaseItem, ConditionalExpr, ParameterExpansion,
    ParameterExpansionSyntax, ParameterOp, Pattern, PatternPart, Position, PrefixMatchKind, Span,
    SubscriptSelector, VarRef, Word, WordPart, WordPartNode };

use super::BacktickEscapedParameter;
use crate::Locator;
use shuck_semantic::{GlobPatternBehavior, PathnameExpansionBehavior, PatternOperatorBehavior};

mod arrays;
mod backticks;
mod command_substitution;
mod expansions;
mod globs;
mod shell_quoting;
mod source_scan;

pub(crate) use arrays::*;
pub(crate) use backticks::{
    backtick_double_escaped_parameter_spans, backtick_escaped_parameter_reference_spans,
    backtick_escaped_parameters, shellcheck_collapsed_backtick_part_span };
pub(crate) use command_substitution::*;
pub(crate) use expansions::*;
pub(crate) use globs::*;
pub(crate) use shell_quoting::*;
pub(crate) use source_scan::*;
