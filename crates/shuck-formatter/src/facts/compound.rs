use rustc_hash::FxHashMap as HashMap;

use shuck_ast::{CompoundCommand, Span};

use super::FactSpan;

#[derive(Debug, Clone, Default)]
pub(super) struct CompoundFacts {
    close_spans: HashMap<FactSpan, Span>,
}

impl CompoundFacts {
    pub(super) fn key(command: &CompoundCommand) -> Option<FactSpan> {
        compound_close_key(command)
    }

    pub(super) fn close_span(&self, command: &CompoundCommand) -> Option<Span> {
        Self::key(command).and_then(|key| self.close_span_for_key(key))
    }

    pub(super) fn close_span_for_span(&self, span: Span) -> Option<Span> {
        self.close_spans.get(&FactSpan::from(span)).copied()
    }

    pub(super) fn close_span_for_key(&self, key: FactSpan) -> Option<Span> {
        self.close_spans.get(&key).copied()
    }

    pub(super) fn insert_close_span(&mut self, key: FactSpan, span: Span) {
        self.close_spans.insert(key, span);
    }

    #[cfg(feature = "benchmarking")]
    pub(super) fn len(&self) -> usize {
        self.close_spans.len()
    }
}

fn compound_close_key(command: &CompoundCommand) -> Option<FactSpan> {
    let span = match command {
        CompoundCommand::If(command) => command.span,
        CompoundCommand::For(command) => match command.syntax {
            shuck_ast::ForSyntax::InDoDone { .. } => command.span,
            shuck_ast::ForSyntax::InDirect { .. } => {
                return None;
            }
        },
        CompoundCommand::ArithmeticFor(command) => command.span,
        CompoundCommand::While(command) => command.span,
        CompoundCommand::Until(command) => command.span,
        CompoundCommand::Case(command) => command.span,
        CompoundCommand::Select(command) => command.span,
        CompoundCommand::Subshell(_)
        | CompoundCommand::BraceGroup(_)
        | CompoundCommand::Arithmetic(_)
        | CompoundCommand::Time(_)
        | CompoundCommand::Conditional(_)
        | CompoundCommand::Coproc(_) => return None,
    };
    Some(FactSpan::from(span))
}
