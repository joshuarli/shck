use super::super::*;

impl<'a> Parser<'a> {
    pub(in crate::parser) fn current_zsh_glob_word_from_source(&mut self) -> Option<Word> {
        None
    }

    pub(in crate::parser) fn token_source_like_word_text(
        &self,
        token: &LexedToken<'a>,
    ) -> Option<Cow<'a, str>> {
        token
            .source_slice(self.input)
            .map(Cow::Borrowed)
            .or_else(|| {
                (token.span.start.offset() <= token.span.end.offset()
                    && token.span.end.offset() <= self.input.len())
                .then(|| {
                    Cow::Borrowed(&self.input[token.span.start.offset()..token.span.end.offset()])
                })
            })
            .or_else(|| token.word_string().map(Cow::Owned))
    }

    pub(in crate::parser) fn current_source_like_word_text(&self) -> Option<Cow<'a, str>> {
        self.current_token_kind
            .filter(|kind| kind.is_word_like())
            .and(self.current_token.as_ref())
            .and_then(|token| self.token_source_like_word_text(token))
    }

    pub(in crate::parser) fn current_source_like_word_text_or_error(
        &self,
        context: &'static str,
    ) -> Result<Cow<'a, str>> {
        self.current_source_like_word_text().ok_or_else(|| {
            self.error(format!(
                "internal parser error: missing source text for {context}"
            ))
        })
    }
}
