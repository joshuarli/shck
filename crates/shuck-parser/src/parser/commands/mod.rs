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
enum ForHeaderSurface {
    In {
        in_span: Option<Span>,
    },
}

