use shuck_ast::BackgroundOperator;
use super::*;

impl<'a> Parser<'a> {
    pub(super) fn parse_function_body_command(
        &mut self,
        allow_bare_compound: bool,
    ) -> Result<Stmt> {
        let compound = match self.current_keyword() {
            Some(Keyword::If) if allow_bare_compound => self.parse_if()?,
            Some(Keyword::For) if allow_bare_compound => self.parse_for()?,            Some(Keyword::While) if allow_bare_compound => self.parse_while()?,
            Some(Keyword::Until) if allow_bare_compound => self.parse_until()?,
            Some(Keyword::Case) if allow_bare_compound => self.parse_case()?,
            Some(Keyword::Select) if allow_bare_compound => self.parse_select()?,
            _ => match self.current_token_kind {
                Some(TokenKind::LeftBrace) => self.parse_brace_group(BraceBodyContext::Function)?,
                Some(TokenKind::LeftParen) => self.parse_subshell()?,
                Some(TokenKind::DoubleLeftBracket) if allow_bare_compound => {
                    self.parse_conditional()?
                }
                Some(TokenKind::DoubleLeftParen) if allow_bare_compound => {
                    if self.looks_like_command_style_double_paren() {
                        self.split_current_double_left_paren();
                        self.parse_subshell()?
                    } else {
                        let checkpoint = self.checkpoint();
                        if let Ok(compound) = self.parse_arithmetic_command() {
                            compound
                        } else {
                            self.restore(checkpoint);
                            self.split_current_double_left_paren();
                            self.parse_subshell()?
                        }
                    }
                }
                _ => {
                    return Err(Error::parse(
                        "expected compound command for function body".to_string(),
                    ));
                }
            },
        };
        let redirects = self.parse_trailing_redirects();
        Ok(Self::lower_non_sequence_command_to_stmt(Command::Compound(
            Box::new(compound),
            redirects,
        )))
    }

    pub(super) fn parse_function_header_entry(&mut self) -> Result<FunctionHeaderEntry> {
        let word = self
            .take_current_function_header_word_and_advance()
            .ok_or_else(|| self.error("expected function name"))?;
        Ok(self.function_header_entry_from_word(word))
    }

    pub(super) fn parse_function_keyword_header_entry(&mut self) -> Result<FunctionHeaderEntry> {
        let word = self
            .take_current_function_header_word_and_advance()
            .or_else(|| self.take_current_function_keyword_name_and_advance())
            .ok_or_else(|| self.error("expected function name"))?;
        Ok(self.function_header_entry_from_word(word))
    }

    pub(super) fn take_current_function_header_word_and_advance(&mut self) -> Option<Word> {
        let span = self.current_span;
        if let Some(token) = self.current_token.clone()
            && let Some(word) = self.simple_word_from_token(&token, span)
        {
            self.advance_past_word(&word);
            return Some(word);
        }

        let token = self.current_token.take()?;
        let word = self.decode_word_from_token(&token, span);
        self.current_token = Some(token);
        if let Some(word) = word.as_ref() {
            self.advance_past_word(word);
        }
        word
    }

    pub(super) fn take_current_function_keyword_name_and_advance(&mut self) -> Option<Word> {
        let text = match self.current_token_kind? {
            TokenKind::DoubleLeftBracket => "[[",
            TokenKind::DoubleRightBracket => "]]",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            _ => return None,
        };
        let word = Word::literal_with_span(text, self.current_span);
        self.advance();
        Some(word)
    }

    pub(super) fn function_header_entry_from_word(&self, word: Word) -> FunctionHeaderEntry {
        let static_name = self.literal_word_text(&word).map(Name::from);
        FunctionHeaderEntry { word, static_name }
    }

    pub(super) fn parse_function_parens_span(&mut self) -> Result<Span> {
        if !self.at(TokenKind::LeftParen) {
            return Err(self.error("expected '(' in function definition"));
        }
        let left_paren_span = self.current_span;
        self.advance();

        if !self.at(TokenKind::RightParen) {
            return Err(Error::parse(
                "expected ')' in function definition".to_string(),
            ));
        }
        let right_paren_span = self.current_span;
        self.advance();
        Ok(left_paren_span.merge(right_paren_span))
    }


    #[allow(dead_code)]
    pub(super) fn parse_single_stmt_command(&mut self) -> Result<Stmt> {
        let mut stmt = self
            .parse_pipeline()?
            .ok_or_else(|| self.error("expected command"))?;

        let Some(kind) = self.current_token_kind else {
            return Ok(stmt);
        };
        let operator = match kind {
            TokenKind::And => Some((Some(BinaryOp::And), None, false)),
            TokenKind::Or => Some((Some(BinaryOp::Or), None, false)),
            TokenKind::Semicolon => Some((None, Some(StmtTerminator::Semicolon), true)),
            TokenKind::Background => Some((
                None,
                Some(StmtTerminator::Background(BackgroundOperator::Plain)),
                true,
            )),
            TokenKind::BackgroundPipe => Some((
                None,
                Some(StmtTerminator::Background(BackgroundOperator::Plain)),
                true,
            )),
            TokenKind::BackgroundBang => Some((
                None,
                Some(StmtTerminator::Background(BackgroundOperator::Plain)),
                true,
            )),
            _ => None,
        };
        let Some((binary_op, terminator, allow_empty_tail)) = operator else {
            return Ok(stmt);
        };
        let operator_span = self.current_span;
        self.advance();

        if let Some(binary_op) = binary_op {
            self.skip_newlines()?;
            if let Some(right) = self.parse_pipeline()? {
                stmt = Self::binary_stmt(stmt, binary_op, operator_span, right);
            }
            return Ok(stmt);
        }

        if allow_empty_tail
            && matches!(
                self.current_token_kind,
                Some(TokenKind::Semicolon | TokenKind::Newline)
            )
        {
            self.advance();
        }

        stmt.terminator = terminator;
        stmt.terminator_span = Some(operator_span);
        Ok(stmt)
    }


    /// Parse function definition with 'function' keyword: function name { body }
    pub(super) fn parse_function_keyword(&mut self) -> Result<Command> {
        self.ensure_function_keyword()?;
        let start_span = self.current_span;
        self.advance(); // consume 'function'
        self.skip_newlines()?;

        let entry = self.parse_function_keyword_header_entry()?;
        let saw_newline_after_name = self.skip_newlines_with_flag()?;
        let (trailing_parens_span, allow_bare_compound) = if self.at(TokenKind::LeftParen) {
            let parens_span = self.parse_function_parens_span()?;
            (Some(parens_span), self.skip_newlines_with_flag()?)
        } else {
            (None, saw_newline_after_name)
        };

        let body = self.parse_function_body_command(allow_bare_compound)?;
        let span = start_span.merge(self.current_span);

        Ok(Command::Function(FunctionDef {
            header: FunctionHeader {
                function_keyword_span: Some(start_span),
                entries: vec![entry],
                trailing_parens_span,
            },
            body: Box::new(body),
            span,
        }))
    }

    /// Parse POSIX-style function definition: name() { body }
    pub(super) fn parse_function_posix(&mut self) -> Result<Command> {
        let start_span = self.current_span;
        let entry = self.parse_function_header_entry()?;
        let trailing_parens_span = self.parse_function_parens_span()?;

        self.finish_parse_function_posix(start_span, entry, trailing_parens_span)
    }

    pub(super) fn finish_parse_function_posix(
        &mut self,
        start_span: Span,
        entry: FunctionHeaderEntry,
        trailing_parens_span: Span,
    ) -> Result<Command> {
        self.skip_newlines()?;
        let body = self.parse_function_body_command(true)?;

        Ok(Command::Function(FunctionDef {
            header: FunctionHeader {
                function_keyword_span: None,
                entries: vec![entry],
                trailing_parens_span: Some(trailing_parens_span),
            },
            body: Box::new(body),
            span: start_span.merge(self.current_span),
        }))
    }



    #[allow(dead_code)]
    pub(super) fn parse_anonymous_paren_function(&mut self) -> Result<Command> {
        Err(self.error("anonymous functions are not supported"))
    }

}
