use super::*;

impl<'a> Parser<'a> {
    pub(super) fn parse_case(&mut self) -> Result<CompoundCommand> {
        let start_span = self.current_span;
        self.push_depth()?;
        self.advance(); // consume 'case'
        self.skip_newlines()?;

        // Get the word to match against
        let word = self.expect_word()?;
        self.skip_newlines()?;

        // Expect 'in'
        self.expect_keyword(Keyword::In)?;
        self.skip_newlines()?;

        // Parse case items
        let mut cases = Vec::new();
        while !self.is_keyword(Keyword::Esac) && self.current_token.is_some() {
            self.skip_newlines()?;
            if self.is_keyword(Keyword::Esac) {
                break;
            }

            let patterns = match self.parse_case_patterns() {
                Ok(patterns) => patterns,
                Err(err) => {
                    self.pop_depth();
                    return Err(err);
                }
            };
            self.skip_newlines()?;

            // Parse commands until ;; or esac
            let body_start = self.current_span.start;
            let mut commands = Vec::new();
            while !self.is_case_terminator()
                && !self.is_keyword(Keyword::Esac)
                && self.current_token.is_some()
            {
                commands.extend(self.parse_command_list_required()?);
                self.skip_newlines()?;
            }

            let (terminator, terminator_span) = self.parse_case_terminator();
            let body_span = Span::from_positions(body_start, self.current_span.start);
            cases.push(CaseItem {
                patterns,
                body: Self::stmt_seq_with_span(body_span, commands),
                terminator,
                terminator_span,
            });
            self.skip_newlines()?;
        }

        // Expect 'esac'
        if !self.is_keyword(Keyword::Esac) {
            self.pop_depth();
            return Err(self.error("expected 'esac'"));
        }
        let esac_span = self.current_span;
        self.advance();

        self.pop_depth();
        Ok(CompoundCommand::Case(Box::new(CaseCommand {
            word,
            cases,
            esac_span,
            span: start_span.merge(self.current_span),
        })))
    }

    pub(super) fn parse_case_patterns(&mut self) -> Result<Vec<Pattern>> {
        self.parse_posix_case_patterns()
    }


    pub(super) fn parse_posix_case_patterns(&mut self) -> Result<Vec<Pattern>> {
        if self.at(TokenKind::LeftParen) {
            self.advance();
        }

        let mut patterns = Vec::new();
        while self.at_word_like() {
            if let Some(word) = self.take_current_word_and_advance() {
                patterns.push(self.pattern_from_word(&word));
            }

            if self.at(TokenKind::Pipe) {
                self.advance();
            } else {
                break;
            }
        }

        if !self.at(TokenKind::RightParen) {
            return Err(self.error("expected ')' after case pattern"));
        }
        self.advance();

        Ok(patterns)
    }







}
