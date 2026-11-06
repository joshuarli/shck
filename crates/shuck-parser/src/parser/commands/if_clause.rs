use super::*;

impl<'a> Parser<'a> {
    pub(super) fn parse_if(&mut self) -> Result<CompoundCommand> {
        let start_span = self.current_span;
        self.push_depth()?;
        self.advance(); // consume 'if'
        self.skip_newlines()?;

        // Parse condition
        let condition_start = self.current_span.start;
        let condition = self.parse_if_condition_until_body_start(false)?;
        let condition_span = Span::from_positions(condition_start, self.current_span.start);
        let condition = Self::stmt_seq_with_span(condition_span, condition);

        let then_span = self.current_span;
        self.expect_keyword(Keyword::Then)?;
        self.skip_newlines()?;

        let then_start = self.current_span.start;
        let then_branch = self.parse_compound_list_until(IF_BODY_TERMINATORS)?;
        let then_branch_span = Span::from_positions(then_start, self.current_span.start);

        let then_branch = if then_branch.is_empty() {
            self.pop_depth();
            return Err(self.error("syntax error: empty then clause"));
        } else {
            Self::stmt_seq_with_span(then_branch_span, then_branch)
        };

        // Parse elif branches
        let mut elif_branches = Vec::new();
        while self.is_keyword(Keyword::Elif) {
            self.advance(); // consume 'elif'
            self.skip_newlines()?;

            let elif_condition_start = self.current_span.start;
            let elif_condition = self.parse_if_condition_until_body_start(false)?;
            let elif_condition_span =
                Span::from_positions(elif_condition_start, self.current_span.start);
            let elif_condition = Self::stmt_seq_with_span(elif_condition_span, elif_condition);

            self.expect_keyword(Keyword::Then)?;
            let elif_body_start = self.current_span.start;
            let elif_body = self.parse_compound_list_until(IF_BODY_TERMINATORS)?;
            let elif_body_span = Span::from_positions(elif_body_start, self.current_span.start);

            if elif_body.is_empty() {
                self.pop_depth();
                return Err(self.error("syntax error: empty elif clause"));
            }

            elif_branches.push((
                elif_condition,
                Self::stmt_seq_with_span(elif_body_span, elif_body),
            ));
        }

        // Parse else branch
        let else_branch = if self.is_keyword(Keyword::Else) {
            self.advance(); // consume 'else'
            self.skip_newlines()?;
            let else_start = self.current_span.start;
            let branch = self.parse_compound_list(Keyword::Fi)?;
            let else_span = Span::from_positions(else_start, self.current_span.start);

            if branch.is_empty() {
                self.pop_depth();
                return Err(self.error("syntax error: empty else clause"));
            } else {
                Some(Self::stmt_seq_with_span(else_span, branch))
            }
        } else {
            None
        };

        if !self.is_keyword(Keyword::Fi) {
            self.pop_depth();
            return Err(self.error("expected 'fi'"));
        }
        let fi_span = self.current_span;
        self.advance();

        self.pop_depth();
        Ok(CompoundCommand::If(Box::new(IfCommand {
            condition,
            then_branch,
            elif_branches,
            else_branch,
            syntax: IfSyntax::ThenFi { then_span, fi_span },
            span: start_span.merge(self.current_span),
        })))
    }
}
