use shuck_ast::File;

use crate::error::Error;

/// Overall outcome of a parse attempt.
///
/// A parse can produce an AST even when recovery diagnostics were needed.
/// Check [`ParseResult::is_ok`] when a caller requires a fully clean parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    /// The parse completed without recovery diagnostics.
    Clean,
    /// The parse completed, but required recovery diagnostics.
    Recovered,
    /// The parse stopped after a terminal error.
    Fatal,
}

/// Additional parser-owned facts that are useful to downstream consumers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyntaxFacts {}

/// A parser diagnostic emitted while recovering from invalid input.
///
/// Diagnostics use Shuck-authored wording and source spans in the parsed input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    /// Human-readable diagnostic message.
    pub message: String,
    /// Source span associated with the diagnostic.
    pub span: shuck_ast::Span,
}

/// The result of parsing a script.
///
/// `ParseResult` always carries the best AST the parser produced. Inspect
/// [`ParseResult::status`] or [`ParseResult::is_ok`] before assuming it was a
/// clean parse.
///
/// This is a parser-owned output type. Its fields remain public for inspection, but consumers
/// should obtain it from [`crate::parser::Parser::parse`] instead of using a struct literal.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ParseResult {
    /// Parsed syntax tree for the file.
    pub file: File,
    /// Recovery diagnostics emitted while producing the AST.
    pub diagnostics: Vec<ParseDiagnostic>,
    /// High-level parse status.
    pub status: ParseStatus,
    /// Terminal parse error, when recovery could not continue.
    pub terminal_error: Option<Error>,
    /// Additional syntax facts collected during parsing.
    pub syntax_facts: SyntaxFacts,
}

impl ParseResult {
    /// Returns `true` when the parse completed without recovery diagnostics.
    pub fn is_ok(&self) -> bool {
        self.status == ParseStatus::Clean
    }

    /// Returns `true` when the parse produced recovery diagnostics or a terminal error.
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Convert this result into a strict parse error.
    pub fn strict_error(&self) -> Error {
        self.terminal_error.clone().unwrap_or_else(|| {
            let Some(diagnostic) = self.diagnostics.first() else {
                panic!("non-clean parse result should include a diagnostic or terminal error");
            };
            Error::parse_at(
                diagnostic.message.clone(),
                diagnostic.span.start.line(),
                diagnostic.span.start.column(),
            )
        })
    }

    /// Return the parse result when it is clean, otherwise panic with the strict error.
    pub fn unwrap(self) -> Self {
        if self.is_ok() {
            self
        } else {
            panic!(
                "called `ParseResult::unwrap()` on a non-clean parse: {}",
                self.strict_error()
            )
        }
    }

    /// Return the strict parse error when the result is not clean, otherwise panic.
    pub fn unwrap_err(self) -> Error {
        if self.is_err() {
            self.strict_error()
        } else {
            panic!("called `ParseResult::unwrap_err()` on a clean parse")
        }
    }

    /// Return the strict parse error when the result is not clean, otherwise panic with
    /// `message`.
    pub fn expect_err(self, message: &str) -> Error {
        if self.is_err() {
            self.strict_error()
        } else {
            panic!("{message}")
        }
    }
}
