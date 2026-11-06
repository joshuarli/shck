use super::super::*;

impl<'a> Parser<'a> {
    pub(in crate::parser) fn parameter_word_part_from_legacy(
        &self,
        part: WordPart,
        part_start: Position,
        part_end: Position,
        source_backed: bool,
    ) -> WordPart {
        let span = Span::from_positions(part_start, part_end);
        let raw_body = self.parameter_raw_body_from_legacy(&part, span, source_backed);
        let raw_body_text = raw_body.slice(self.input).to_string();

        let syntax = match part {
            WordPart::ParameterExpansion {
                reference,
                operator,
                operand,
                operand_word_ast,
                colon_variant,
            } => Some(BourneParameterExpansion::Operation {
                reference: *reference,
                operator: self.enrich_parameter_operator(operator),
                operand: operand.map(|operand| *operand),
                operand_word_ast,
                colon_variant,
            }),
            WordPart::Length(reference) | WordPart::ArrayLength(reference) => {
                Some(BourneParameterExpansion::Length {
                    reference: *reference,
                })
            }
            WordPart::ArrayAccess(reference) => Some(BourneParameterExpansion::Access {
                reference: *reference,
            }),
            WordPart::ArrayIndices(reference) => Some(BourneParameterExpansion::Indices {
                reference: *reference,
            }),
            WordPart::Substring {
                reference,
                offset,
                offset_ast,
                offset_word_ast,
                length,
                length_ast,
                length_word_ast,
            }
            | WordPart::ArraySlice {
                reference,
                offset,
                offset_ast,
                offset_word_ast,
                length,
                length_ast,
                length_word_ast,
            } => Some(BourneParameterExpansion::Slice {
                reference: *reference,
                offset: *offset,
                offset_ast,
                offset_word_ast,
                length: length.map(|length| *length),
                length_ast,
                length_word_ast,
            }),
            WordPart::IndirectExpansion {
                reference,
                operator,
                operand,
                operand_word_ast,
                colon_variant,
            } => Some(BourneParameterExpansion::Indirect {
                reference: *reference,
                operator: operator.map(|operator| self.enrich_parameter_operator(operator)),
                operand: operand.map(|operand| *operand),
                operand_word_ast,
                colon_variant,
            }),
            WordPart::PrefixMatch { prefix, kind } => {
                Some(BourneParameterExpansion::PrefixMatch { prefix, kind })
            }
            WordPart::Transformation {
                reference,
                operator,
            } => Some(BourneParameterExpansion::Transformation {
                reference: *reference,
                operator,
            }),
            WordPart::Variable(name) if raw_body_text == name.as_str() => {
                Some(BourneParameterExpansion::Access {
                    reference: self.parameter_var_ref(
                        part_start,
                        "${",
                        name.as_str(),
                        None,
                        part_end,
                    ),
                })
            }
            other => return other,
        };

        let Some(syntax) = syntax else {
            unreachable!("matched Some above");
        };
        WordPart::Parameter(Box::new(ParameterExpansion {
            syntax: ParameterExpansionSyntax::Bourne(syntax),
            span,
            raw_body,
        }))
    }

    pub(in crate::parser) fn enrich_parameter_operator(
        &self,
        operator: Box<ParameterOp>,
    ) -> Box<ParameterOp> {
        match *operator {
            ParameterOp::ReplaceFirst {
                pattern,
                replacement,
                ..
            } => Box::new(ParameterOp::ReplaceFirst {
                pattern,
                replacement_word_ast: Box::new(self.parse_source_text_as_word(&replacement)),
                replacement,
            }),
            ParameterOp::ReplaceAll {
                pattern,
                replacement,
                ..
            } => Box::new(ParameterOp::ReplaceAll {
                pattern,
                replacement_word_ast: Box::new(self.parse_source_text_as_word(&replacement)),
                replacement,
            }),
            ParameterOp::UseDefault
            | ParameterOp::AssignDefault
            | ParameterOp::UseReplacement
            | ParameterOp::Error
            | ParameterOp::RemovePrefixShort { .. }
            | ParameterOp::RemovePrefixLong { .. }
            | ParameterOp::RemoveSuffixShort { .. }
            | ParameterOp::RemoveSuffixLong { .. }
            | ParameterOp::UpperFirst
            | ParameterOp::UpperAll
            | ParameterOp::LowerFirst
            | ParameterOp::LowerAll => operator,
        }
    }

    pub(in crate::parser) fn parameter_raw_body_from_legacy(
        &self,
        part: &WordPart,
        span: Span,
        source_backed: bool,
    ) -> SourceText {
        if source_backed && span.end.offset() <= self.input.len() {
            let syntax = span.slice(self.input);
            if let Some(body) = syntax
                .strip_prefix("${")
                .and_then(|syntax| syntax.strip_suffix('}'))
            {
                let start = span.start.advanced_by("${");
                let end = start.advanced_by(body);
                return SourceText::source(Span::from_positions(start, end));
            }
        }

        let mut syntax = String::new();
        self.push_word_part_syntax(&mut syntax, part, span);
        let body = syntax
            .strip_prefix("${")
            .and_then(|syntax| syntax.strip_suffix('}'))
            .unwrap_or(syntax.as_str())
            .to_string();
        SourceText::from(body)
    }
    pub(in crate::parser) fn zsh_parameter_word_part(
        &mut self,
        raw_body: SourceText,
        part_start: Position,
        part_end: Position,
    ) -> WordPart {
        let reference = self.parse_loose_var_ref(raw_body.slice(self.input), raw_body.span().start);
        WordPart::Parameter(Box::new(ParameterExpansion {
            syntax: ParameterExpansionSyntax::Bourne(BourneParameterExpansion::Access { reference }),
            span: Span::from_positions(part_start, part_end),
            raw_body,
        }))
    }

    #[allow(dead_code)]
    pub(in crate::parser) fn maybe_parse_loose_var_ref_target(
        &self,
        text: &str,
        base: Position,
    ) -> Option<VarRef> {
        let trimmed = text.trim();
        Self::looks_like_plain_parameter_access(trimmed)
            .then(|| self.parse_loose_var_ref(text, base))
    }

    #[allow(dead_code)]
    pub(in crate::parser) fn is_plain_special_parameter_name(name: &str) -> bool {
        matches!(name, "#" | "$" | "!" | "*" | "@" | "?" | "-") || name == "0"
    }

    #[allow(dead_code)]
    pub(in crate::parser) fn is_plain_parameter_access_name(name: &str) -> bool {
        Self::is_valid_identifier(name)
            || name.bytes().all(|byte| byte.is_ascii_digit())
            || Self::is_plain_special_parameter_name(name)
    }

    #[allow(dead_code)]
    pub(in crate::parser) fn looks_like_plain_parameter_access(text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return false;
        }

        let name = if let Some(open) = trimmed.find('[') {
            if !trimmed.ends_with(']') {
                return false;
            }
            &trimmed[..open]
        } else {
            trimmed
        };

        Self::is_plain_parameter_access_name(name)
            || name
                .strip_prefix('+')
                .is_some_and(Self::is_plain_parameter_access_name)
    }
    pub(in crate::parser) fn parse_loose_var_ref(&self, text: &str, base: Position) -> VarRef {
        let trimmed = text.trim();
        let base = base.advanced_by(&text[..text.len() - text.trim_start().len()]);
        let span = Span::from_positions(base, base.advanced_by(trimmed));
        if let Some(open) = trimmed.find('[')
            && trimmed.ends_with(']')
        {
            let name = &trimmed[..open];
            let subscript_text = &trimmed[open + 1..trimmed.len() - 1];
            let subscript_start = base.advanced_by(&trimmed[..open + 1]);
            let subscript = self.subscript_from_source_text(
                self.source_text_from_str(
                    subscript_text,
                    subscript_start,
                    subscript_start.advanced_by(subscript_text),
                ),
                None,
                SubscriptInterpretation::Contextual,
            );
            return VarRef {
                name: Name::from(name),
                name_span: Span::from_positions(base, base.advanced_by(name)),
                subscript: Some(Box::new(subscript)),
                span,
            };
        }

        VarRef {
            name: Name::from(trimmed),
            name_span: span,
            subscript: None,
            span,
        }
    }

    #[allow(dead_code)]
    pub(in crate::parser) fn find_matching_parameter_end(&self, text: &str) -> Option<usize> {
        let mut depth = 0_i32;
        let mut chars = text.char_indices().peekable();

        while let Some((index, ch)) = chars.next() {
            match ch {
                '$' if chars.peek().is_some_and(|(_, next)| *next == '{') => {
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(index + ch.len_utf8());
                    }
                }
                _ => {}
            }
        }

        None
    }



}
