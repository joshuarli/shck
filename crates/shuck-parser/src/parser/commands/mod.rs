use super::*;
use smallvec::SmallVec;

mod case;
mod compound;
mod conditionals;
mod functions;
mod if_clause;
mod lists;
mod loops;
mod simple;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum ForHeaderSurface {
    In {
        in_span: Option<Span>,
    },
    #[allow(dead_code)]
    Paren {
        left_paren_span: Span,
        right_paren_span: Span,
    },
}

