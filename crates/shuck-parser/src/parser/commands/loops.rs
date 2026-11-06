use super::*;

impl<'a> Parser<'a> {
    pub(super) fn parse_for(&mut self) -> Result<CompoundCommand> {
        let start_span = self.current_span;
        self.push_depth()?;
        self.advance(); // consume 'for'
        self.skip_newlines()?;

        // Check for C-style for loop: for ((init; cond; step))
        if self.at(TokenKind::DoubleLeftParen) {
            let result = self.parse_arithmetic_for_inner(start_span);
            self.pop_depth();
            return result;
        }

        let targets = match self.parse_for_targets(false) {
            Ok(targets) => targets,
            Err(error) => {
                self.pop_depth();
                return Err(error);
            }
        };

        let (words, header) = if self.is_keyword(Keyword::In) {
            let in_span = self.current_span;
            self.advance();

            let (words, saw_separator) = self.parse_for_word_list_until_body_separator()?;
            if !saw_separator {
                self.pop_depth();
                return Err(self.error("expected ';' or newline before for loop body"));
            }
            (
                Some(words),
                ForHeaderSurface::In {
                    in_span: Some(in_span),
                },
            )
        } else {
            if self.at(TokenKind::Semicolon) {
                self.advance();
            }
            self.skip_newlines()?;
            (None, ForHeaderSurface::In { in_span: None })
        };

        let (body, syntax, end_span) = match header {
            ForHeaderSurface::In { in_span } => {
                if !self.is_keyword(Keyword::Do) {
                    self.pop_depth();
                    return Err(self.error("expected 'do'"));
                }
                let do_span = self.current_span;
                self.advance();
                self.skip_newlines()?;

                let body_start = self.current_span.start;
                let body = self.parse_compound_list(Keyword::Done)?;
                let body_span = Span::from_positions(body_start, self.current_span.start);
                if body.is_empty() {
                    self.pop_depth();
                    return Err(self.error("syntax error: empty for loop body"));
                }
                if !self.is_keyword(Keyword::Done) {
                    self.pop_depth();
                    return Err(self.error("expected 'done'"));
                }
                let done_span = self.current_span;
                self.advance();
                let body = Self::stmt_seq_with_span(body_span, body);
                (
                    body,
                    ForSyntax::InDoDone {
                        in_span,
                        do_span,
                        done_span,
                    },
                    done_span,
                )
            }
            ForHeaderSurface::Paren { .. } => {
                self.pop_depth();
                return Err(self.error("expected 'do'"));
            }
        };

        self.pop_depth();
        Ok(CompoundCommand::For(Box::new(ForCommand {
            targets: targets.into_vec(),
            words: words.map(SmallVec::into_vec),
            body,
            syntax,
            span: start_span.merge(end_span),
        })))
    }

    pub(super) fn parse_for_targets(
        &mut self,
        _allow_zsh_targets: bool,
    ) -> Result<SmallVec<[ForTarget; 1]>> {
        let first_target = self
            .current_for_target(false)
            .ok_or_else(|| Error::parse("expected variable name in for loop".to_string()))?;
        let first_word = first_target.word.clone();
        self.advance_past_word(&first_word);
        Ok(SmallVec::from_vec(vec![first_target]))
    }

    pub(super) fn current_for_target(&mut self, allow_digits: bool) -> Option<ForTarget> {
        let name = self.current_word_str().and_then(|name| {
            (Self::is_valid_identifier(name)
                || (allow_digits && name.bytes().all(|byte| byte.is_ascii_digit())))
            .then(|| Name::from(name))
        });
        let word = self.current_word()?;
        Some(ForTarget {
            span: word.span,
            word,
            name,
        })
    }

    pub(super) fn parse_for_word_list_until_body_separator(
        &mut self,
    ) -> Result<(SmallVec<[Word; 2]>, bool)> {
        let mut words = SmallVec::<[Word; 2]>::new();
        loop {
            match self.current_token_kind {
                Some(kind) if kind.is_word_like() => {
                    let word = self
                        .take_current_word_and_advance()
                        .ok_or_else(|| self.error("expected for word"))?;
                    words.push(word);
                }
                Some(TokenKind::Semicolon) => {
                    self.advance();
                    self.skip_newlines()?;
                    return Ok((words, true));
                }
                Some(TokenKind::Newline) => {
                    self.skip_newlines()?;
                    return Ok((words, true));
                }
                _ => return Ok((words, false)),
            }
        }
    }

    /// Parse select loop: select var in list; do body; done
    pub(super) fn parse_select(&mut self) -> Result<CompoundCommand> {
        self.ensure_select_loop()?;
        let start_span = self.current_span;
        self.push_depth()?;
        self.advance(); // consume 'select'
        self.skip_newlines()?;

        // Expect variable name
        let (variable, variable_span) = match self.current_name_token() {
            Some(pair) => pair,
            _ => {
                self.pop_depth();
                return Err(Error::parse("expected variable name in select".to_string()));
            }
        };
        self.advance();

        // Expect 'in' keyword
        if !self.is_keyword(Keyword::In) {
            self.pop_depth();
            return Err(Error::parse("expected 'in' in select".to_string()));
        }
        self.advance(); // consume 'in'

        // Parse word list until do/newline/;
        let mut words = SmallVec::<[Word; 2]>::new();
        loop {
            match self.current_token_kind {
                _ if self.current_keyword() == Some(Keyword::Do) => break,
                Some(kind) if kind.is_word_like() => {
                    if let Some(word) = self.take_current_word_and_advance() {
                        words.push(word);
                    }
                }
                Some(TokenKind::Newline | TokenKind::Semicolon) => {
                    self.advance();
                    break;
                }
                _ => break,
            }
        }

        self.skip_newlines()?;

        // Expect 'do'
        self.expect_keyword(Keyword::Do)?;
        self.skip_newlines()?;

        // Parse body
        let body_start = self.current_span.start;
        let body = self.parse_compound_list(Keyword::Done)?;
        let body_span = Span::from_positions(body_start, self.current_span.start);

        // Bash requires at least one command in loop body
        if body.is_empty() {
            self.pop_depth();
            return Err(self.error("syntax error: empty select loop body"));
        }
        let body = Self::stmt_seq_with_span(body_span, body);

        // Expect 'done'
        if !self.is_keyword(Keyword::Done) {
            self.pop_depth();
            return Err(self.error("expected 'done'"));
        }
        let done_span = self.current_span;
        self.advance();

        self.pop_depth();
        Ok(CompoundCommand::Select(Box::new(SelectCommand {
            variable,
            variable_span,
            words: words.into_vec(),
            body,
            done_span,
            span: start_span.merge(self.current_span),
        })))
    }

    /// Parse C-style arithmetic for loop inner: for ((init; cond; step)); do body; done
    /// Note: depth tracking is done by parse_for which calls this
    pub(super) fn parse_arithmetic_for_inner(
        &mut self,
        start_span: Span,
    ) -> Result<CompoundCommand> {
        self.ensure_arithmetic_for()?;
        let left_paren_span = self.current_span;
        self.advance(); // consume '(('

        let mut paren_depth = 0_i32;
        let mut segment_start = left_paren_span.end;
        let mut init_span = None;
        let mut first_semicolon_span = None;
        let mut condition_span = None;
        let mut second_semicolon_span = None;

        let right_paren_span = loop {
            match self.current_token_kind {
                Some(TokenKind::DoubleLeftParen) => {
                    paren_depth += 2;
                    self.advance();
                }
                Some(TokenKind::LeftParen) => {
                    paren_depth += 1;
                    self.advance();
                }
                Some(TokenKind::ProcessSubIn) | Some(TokenKind::ProcessSubOut) => {
                    paren_depth += 1;
                    self.advance();
                }
                Some(TokenKind::DoubleRightParen) => {
                    if paren_depth == 0 {
                        let right_paren_span = self.current_span;
                        self.advance();
                        break right_paren_span;
                    }
                    if paren_depth == 1 {
                        break self.split_nested_arithmetic_close("arithmetic for header")?;
                    }
                    paren_depth -= 2;
                    self.advance();
                }
                Some(TokenKind::RightParen) => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                    self.advance();
                }
                Some(TokenKind::DoubleSemicolon) if paren_depth == 0 => {
                    let (first_span, second_span) = Self::split_double_semicolon(self.current_span);
                    Self::record_arithmetic_for_separator(
                        first_span,
                        &mut segment_start,
                        &mut init_span,
                        &mut first_semicolon_span,
                        &mut condition_span,
                        &mut second_semicolon_span,
                    )?;
                    Self::record_arithmetic_for_separator(
                        second_span,
                        &mut segment_start,
                        &mut init_span,
                        &mut first_semicolon_span,
                        &mut condition_span,
                        &mut second_semicolon_span,
                    )?;
                    self.advance();
                }
                Some(TokenKind::Semicolon) if paren_depth == 0 => {
                    Self::record_arithmetic_for_separator(
                        self.current_span,
                        &mut segment_start,
                        &mut init_span,
                        &mut first_semicolon_span,
                        &mut condition_span,
                        &mut second_semicolon_span,
                    )?;
                    self.advance();
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    return Err(Error::parse(
                        "unexpected end of input in for loop".to_string(),
                    ));
                }
            }
        };

        let first_semicolon_span = first_semicolon_span
            .ok_or_else(|| Error::parse("expected ';' in arithmetic for header".to_string()))?;
        let second_semicolon_span = second_semicolon_span.ok_or_else(|| {
            Error::parse("expected second ';' in arithmetic for header".to_string())
        })?;
        let step_span = Self::optional_span(segment_start, right_paren_span.start);
        let init_ast =
            self.parse_explicit_arithmetic_span(init_span, "invalid arithmetic for init")?;
        let condition_ast = self
            .parse_explicit_arithmetic_span(condition_span, "invalid arithmetic for condition")?;
        let step_ast =
            self.parse_explicit_arithmetic_span(step_span, "invalid arithmetic for step")?;

        self.skip_newlines()?;

        // Skip optional semicolon after ))
        if self.at(TokenKind::Semicolon) {
            self.advance();
        }
        self.skip_newlines()?;

        let (body, done_span, end_span) = if self.at(TokenKind::LeftBrace) {
            let body = self.parse_brace_group(BraceBodyContext::Ordinary)?;
            let span = Self::compound_span(&body);
            (
                Self::stmt_seq_with_span(
                    span,
                    vec![Self::lower_non_sequence_command_to_stmt(Command::Compound(
                        Box::new(body),
                        SmallVec::<[Redirect; 1]>::new(),
                    ))],
                ),
                None,
                self.current_span,
            )
        } else {
            // Expect 'do'
            self.expect_keyword(Keyword::Do)?;
            self.skip_newlines()?;

            // Parse body
            let body_start = self.current_span.start;
            let body = self.parse_compound_list(Keyword::Done)?;
            let body_span = Span::from_positions(body_start, self.current_span.start);

            // Bash requires at least one command in loop body
            if body.is_empty() {
                return Err(self.error("syntax error: empty for loop body"));
            }

            // Expect 'done'
            if !self.is_keyword(Keyword::Done) {
                return Err(self.error("expected 'done'"));
            }
            let done_span = self.current_span;
            self.advance();
            (
                Self::stmt_seq_with_span(body_span, body),
                Some(done_span),
                done_span,
            )
        };

        Ok(CompoundCommand::ArithmeticFor(Box::new(
            ArithmeticForCommand {
                left_paren_span,
                init_span,
                init_ast,
                first_semicolon_span,
                condition_span,
                condition_ast,
                second_semicolon_span,
                step_span,
                step_ast,
                right_paren_span,
                done_span,
                body,
                span: start_span.merge(end_span),
            },
        )))
    }

    /// Parse a while loop
    pub(super) fn parse_while(&mut self) -> Result<CompoundCommand> {
        let start_span = self.current_span;
        self.push_depth()?;
        self.advance(); // consume 'while'
        self.skip_newlines()?;

        // Parse condition
        let condition_start = self.current_span.start;
        let condition = self.parse_loop_condition_until_body_start(false)?;
        let condition_span = Span::from_positions(condition_start, self.current_span.start);
        let condition = Self::stmt_seq_with_span(condition_span, condition);

        self.expect_keyword(Keyword::Do)?;
        self.skip_newlines()?;

        let body_start = self.current_span.start;
        let body = self.parse_compound_list(Keyword::Done)?;
        let body_span = Span::from_positions(body_start, self.current_span.start);

        if body.is_empty() {
            self.pop_depth();
            return Err(self.error("syntax error: empty while loop body"));
        }
        let body = Self::stmt_seq_with_span(body_span, body);

        if !self.is_keyword(Keyword::Done) {
            self.pop_depth();
            return Err(self.error("expected 'done'"));
        }
        let done_span = self.current_span;
        self.advance();

        self.pop_depth();
        Ok(CompoundCommand::While(Box::new(WhileCommand {
            condition,
            body,
            done_span: Some(done_span),
            span: start_span.merge(self.current_span),
        })))
    }

    /// Parse an until loop
    pub(super) fn parse_until(&mut self) -> Result<CompoundCommand> {
        let start_span = self.current_span;
        self.push_depth()?;
        self.advance(); // consume 'until'
        self.skip_newlines()?;

        // Parse condition
        let condition_start = self.current_span.start;
        let condition = self.parse_loop_condition_until_body_start(false)?;
        let condition_span = Span::from_positions(condition_start, self.current_span.start);
        let condition = Self::stmt_seq_with_span(condition_span, condition);

        self.expect_keyword(Keyword::Do)?;
        self.skip_newlines()?;

        let body_start = self.current_span.start;
        let body = self.parse_compound_list(Keyword::Done)?;
        let body_span = Span::from_positions(body_start, self.current_span.start);

        if body.is_empty() {
            self.pop_depth();
            return Err(self.error("syntax error: empty until loop body"));
        }
        let body = Self::stmt_seq_with_span(body_span, body);

        if !self.is_keyword(Keyword::Done) {
            self.pop_depth();
            return Err(self.error("expected 'done'"));
        }
        let done_span = self.current_span;
        self.advance();

        self.pop_depth();
        Ok(CompoundCommand::Until(Box::new(UntilCommand {
            condition,
            body,
            done_span: Some(done_span),
            span: start_span.merge(self.current_span),
        })))
    }
}
