use super::*;

impl<'a> Parser<'a> {
    pub(super) fn parse_time(&mut self) -> Result<CompoundCommand> {
        let start_span = self.current_span;
        self.advance(); // consume 'time'
        self.skip_newlines()?;

        // Check for -p flag (POSIX format)
        let posix_format = if self.at(TokenKind::Word) && self.current_word_str() == Some("-p") {
            self.advance();
            self.skip_newlines()?;
            true
        } else {
            false
        };

        // Parse the command to time (if any)
        // time with no command is valid in bash (just outputs timing header)
        let command = self.parse_pipeline()?.map(Box::new);

        Ok(CompoundCommand::Time(Box::new(TimeCommand {
            posix_format,
            command,
            span: start_span.merge(self.current_span),
        })))
    }

    /// Parse a coproc command: `coproc [NAME] command`
    ///
    /// If the token after `coproc` is a simple word followed by a compound
    /// command (`{`, `(`, `while`, `for`, etc.), it is treated as the coproc
    /// name. Otherwise the command starts immediately and the default name
    /// "COPROC" is used.
    pub(super) fn parse_coproc(&mut self) -> Result<CompoundCommand> {
        self.ensure_coproc()?;
        let start_span = self.current_span;
        self.advance(); // consume 'coproc'
        self.skip_newlines()?;

        // Determine if next token is a NAME (simple word that is NOT a compound-
        // command keyword and is followed by a compound command start).
        let (name, name_span) = if self.at(TokenKind::Word) {
            if let Some(word) = self.current_word_str() {
                let word = word.to_string();
                let word_span = self.current_span;
                let is_compound_keyword = matches!(
                    word.as_str(),
                    "if" | "for" | "while" | "until" | "case" | "select" | "time" | "coproc"
                );
                let next_is_compound_start = matches!(
                    self.peek_next_kind(),
                    Some(TokenKind::LeftBrace | TokenKind::LeftParen)
                );
                if !is_compound_keyword && next_is_compound_start {
                    self.advance(); // consume the NAME
                    self.skip_newlines()?;
                    (Name::from(word), Some(word_span))
                } else {
                    (Name::new_static("COPROC"), None)
                }
            } else {
                (Name::new_static("COPROC"), None)
            }
        } else {
            (Name::new_static("COPROC"), None)
        };

        // Parse the command body (could be simple, compound, or pipeline)
        let body = self.parse_pipeline()?;
        let body = body.ok_or_else(|| self.error("coproc: missing command"))?;

        Ok(CompoundCommand::Coproc(Box::new(CoprocCommand {
            name,
            name_span,
            body: Box::new(body),
            span: start_span.merge(self.current_span),
        })))
    }

    /// Check if current token is a case terminator
    pub(super) fn is_case_terminator(&self) -> bool {
        matches!(
            self.current_token_kind,
            Some(TokenKind::DoubleSemicolon | TokenKind::SemiAmp | TokenKind::DoubleSemiAmp)
        )
    }

    /// Parse case terminator: `;;` (break), `;&` (fallthrough),
    /// `;;&` (continue matching)
    pub(super) fn parse_case_terminator(&mut self) -> (CaseTerminator, Option<Span>) {
        match self.current_token_kind {
            Some(TokenKind::SemiAmp) => {
                let span = self.current_span;
                self.advance();
                (CaseTerminator::FallThrough, Some(span))
            }
            Some(TokenKind::DoubleSemiAmp) => {
                let span = self.current_span;
                self.advance();
                (CaseTerminator::Continue, Some(span))
            }
            Some(TokenKind::DoubleSemicolon) => {
                let span = self.current_span;
                self.advance();
                (CaseTerminator::Break, Some(span))
            }
            _ => (CaseTerminator::Break, None),
        }
    }

    /// Parse a subshell (commands in parentheses)
    pub(super) fn parse_subshell(&mut self) -> Result<CompoundCommand> {
        self.push_depth()?;
        self.advance(); // consume '('
        self.skip_newlines()?;

        let body_start = self.current_span.start;
        let mut commands = Vec::new();
        while !matches!(
            self.current_token_kind,
            Some(TokenKind::RightParen | TokenKind::DoubleRightParen) | None
        ) {
            self.skip_newlines()?;
            if matches!(
                self.current_token_kind,
                Some(TokenKind::RightParen | TokenKind::DoubleRightParen)
            ) {
                break;
            }
            commands.extend(self.parse_command_list_required()?);
        }

        if self.at(TokenKind::DoubleRightParen) {
            // `))` at end of nested subshells: consume as single `)`, leave `)` for parent
            self.set_current_kind(TokenKind::RightParen, self.current_span);
        } else if !self.at(TokenKind::RightParen) {
            self.pop_depth();
            return Err(Error::parse("expected ')' to close subshell".to_string()));
        } else {
            self.advance(); // consume ')'
        }

        self.pop_depth();
        Ok(CompoundCommand::Subshell(Self::stmt_seq_with_span(
            Span::from_positions(body_start, self.current_span.start),
            commands,
        )))
    }

    /// Parse a brace group
    pub(super) fn parse_brace_group(
        &mut self,
        context: BraceBodyContext,
    ) -> Result<CompoundCommand> {
        let (compound, _) = self.parse_brace_group_with_close_span(context)?;
        Ok(compound)
    }

    pub(super) fn parse_brace_group_with_close_span(
        &mut self,
        _context: BraceBodyContext,
    ) -> Result<(CompoundCommand, Span)> {
        self.push_depth()?;
        let (body, _left_brace_span, right_brace_span) =
            self.parse_brace_enclosed_stmt_seq("syntax error: empty brace group", _context)?;

        self.pop_depth();
        Ok((CompoundCommand::BraceGroup(body), right_brace_span))
    }

    pub(super) fn parse_brace_enclosed_stmt_seq(
        &mut self,
        empty_error: &str,
        context: BraceBodyContext,
    ) -> Result<(StmtSeq, Span, Span)> {
        let left_brace_span = self.current_span;
        self.advance();
        self.brace_group_depth += 1;
        self.brace_body_stack.push(context);
        self.skip_command_separators()?;

        let body_start = self.current_span.start;
        let mut commands = Vec::new();
        while !matches!(self.current_token_kind, Some(TokenKind::RightBrace) | None) {
            self.skip_command_separators()?;
            if self.at(TokenKind::RightBrace) {
                break;
            }
            commands.extend(self.parse_command_list_required()?);
        }

        if !self.at(TokenKind::RightBrace) {
            self.brace_body_stack.pop();
            self.brace_group_depth -= 1;
            return Err(Error::parse(
                "expected '}' to close brace group".to_string(),
            ));
        }

        if commands.is_empty() {
            self.brace_body_stack.pop();
            self.brace_group_depth -= 1;
            return Err(self.error(empty_error));
        }

        let right_brace_span = self.current_span;
        self.advance();
        self.brace_body_stack.pop();
        self.brace_group_depth -= 1;
        Ok((
            Self::stmt_seq_with_span(
                Span::from_positions(body_start, right_brace_span.start),
                commands,
            ),
            left_brace_span,
            right_brace_span,
        ))
    }

    pub(super) fn parse_if_condition_until_body_start(
        &mut self,
        allow_brace_body: bool,
    ) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::with_capacity(2);

        loop {
            self.skip_newlines()?;

            if self.at(TokenKind::Semicolon) {
                let checkpoint = self.checkpoint();
                self.advance();
                if let Err(error) = self.skip_newlines() {
                    self.restore(checkpoint);
                    return Err(error);
                }
                if self.is_keyword(Keyword::Then)
                    || (allow_brace_body && !stmts.is_empty() && self.at(TokenKind::LeftBrace))
                {
                    break;
                }
                self.restore(checkpoint);
            }

            if self.is_keyword(Keyword::Then)
                || (allow_brace_body && !stmts.is_empty() && self.at(TokenKind::LeftBrace))
            {
                break;
            }

            if self.current_token.is_none() {
                break;
            }

            let command_stmts = self.parse_command_list_required()?;
            self.apply_stmt_list_effects(&command_stmts);
            stmts.extend(command_stmts);
        }

        Ok(stmts)
    }

    pub(super) fn parse_loop_condition_until_body_start(
        &mut self,
        allow_brace_body: bool,
    ) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::with_capacity(2);

        loop {
            self.skip_newlines()?;

            if self.is_keyword(Keyword::Do)
                || (allow_brace_body && !stmts.is_empty() && self.at(TokenKind::LeftBrace))
            {
                break;
            }

            if self.current_token.is_none() {
                break;
            }

            let command_stmts = self.parse_command_list_required()?;
            self.apply_stmt_list_effects(&command_stmts);
            stmts.extend(command_stmts);
        }

        Ok(stmts)
    }

    #[allow(dead_code)]
    pub(super) fn has_recorded_comment_between(
        &self,
        start_offset: usize,
        end_offset: usize,
    ) -> bool {
        self.comments.iter().any(|comment| {
            let comment_start = usize::from(comment.range.start());
            comment_start >= start_offset && comment_start < end_offset
        })
    }

    #[allow(dead_code)]
    pub(super) fn rebase_nested_parse_error(&self, error: Error, base: Position) -> Error {
        let Error::Parse {
            message,
            line,
            column,
        } = error;

        if line == 0 {
            return Error::parse(message);
        }

        let rebased_line = base.line() + line.saturating_sub(1);
        let rebased_column = if line == 1 {
            base.column() + column.saturating_sub(1)
        } else {
            column
        };

        Error::parse_at(message, rebased_line, rebased_column)
    }

    pub(super) fn should_consume_right_brace_as_literal_argument(
        &mut self,
        _next_kind_after_right_brace: Option<TokenKind>,
    ) -> bool {
        self.current_token_has_leading_whitespace()
    }
}
