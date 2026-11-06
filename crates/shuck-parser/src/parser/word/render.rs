use super::super::*;

impl<'a> Parser<'a> {
    pub(in crate::parser) fn word_syntax_is_source_backed(&self, word: &Word) -> bool {
        word.span.end.offset() <= self.input.len()
            && word
                .parts
                .first()
                .is_none_or(|part| part.span.start == word.span.start)
            && word
                .parts
                .last()
                .is_none_or(|part| part.span.end == word.span.end)
            && word
                .parts
                .iter()
                .all(|part| self.word_part_syntax_is_source_backed(&part.kind, part.span))
    }

    pub(in crate::parser) fn word_part_syntax_is_source_backed(
        &self,
        part: &WordPart,
        span: Span,
    ) -> bool {
        span.end.offset() <= self.input.len()
            && match part {
                WordPart::Literal(text) => text.is_source_backed(),
                WordPart::SingleQuoted { value, .. } => value.is_source_backed(),
                WordPart::DoubleQuoted { parts, .. } => parts
                    .iter()
                    .all(|part| self.word_part_syntax_is_source_backed(&part.kind, part.span)),
                WordPart::Variable(_)
                | WordPart::CommandSubstitution { .. }
                | WordPart::ProcessSubstitution { .. }
                | WordPart::PrefixMatch { .. } => true,
                WordPart::ArithmeticExpansion { expression, .. } => expression.is_source_backed(),
                WordPart::Parameter(parameter) => parameter.raw_body.is_source_backed(),
                WordPart::ParameterExpansion {
                    reference,
                    operator,
                    operand,
                    ..
                } => {
                    reference.is_source_backed()
                        && self.parameter_operator_is_source_backed(operator)
                        && operand.as_deref().is_none_or(SourceText::is_source_backed)
                }
                WordPart::Length(reference)
                | WordPart::ArrayAccess(reference)
                | WordPart::ArrayLength(reference)
                | WordPart::ArrayIndices(reference)
                | WordPart::Transformation { reference, .. } => reference.is_source_backed(),
                WordPart::Substring {
                    reference,
                    offset,
                    length,
                    ..
                }
                | WordPart::ArraySlice {
                    reference,
                    offset,
                    length,
                    ..
                } => {
                    reference.is_source_backed()
                        && offset.is_source_backed()
                        && length.as_deref().is_none_or(SourceText::is_source_backed)
                }
                WordPart::IndirectExpansion {
                    reference,
                    operator,
                    operand,
                    ..
                } => {
                    reference.is_source_backed()
                        && operator.is_none()
                        && operand.as_deref().is_none_or(SourceText::is_source_backed)
                }
            }
    }

    pub(in crate::parser) fn parameter_operator_is_source_backed(
        &self,
        operator: &ParameterOp,
    ) -> bool {
        match operator {
            ParameterOp::RemovePrefixShort { pattern }
            | ParameterOp::RemovePrefixLong { pattern }
            | ParameterOp::RemoveSuffixShort { pattern }
            | ParameterOp::RemoveSuffixLong { pattern } => pattern.is_source_backed(),
            ParameterOp::ReplaceFirst {
                pattern,
                replacement,
                ..
            }
            | ParameterOp::ReplaceAll {
                pattern,
                replacement,
                ..
            } => pattern.is_source_backed() && replacement.is_source_backed(),
            _ => true,
        }
    }

    pub(in crate::parser) fn word_part_syntax_text<'b>(
        &'b self,
        part: &'b WordPartNode,
    ) -> Cow<'b, str> {
        if self.word_part_syntax_is_source_backed(&part.kind, part.span) {
            Cow::Borrowed(part.span.slice(self.input))
        } else {
            let mut syntax = String::new();
            self.push_word_part_syntax(&mut syntax, &part.kind, part.span);
            Cow::Owned(syntax)
        }
    }

    pub(in crate::parser) fn compound_array_inner_text<'b>(
        &'b self,
        word: &'b Word,
    ) -> Option<(Cow<'b, str>, Position)> {
        let first = word.parts.first()?;
        let last = word.parts.last()?;
        let first_syntax = self.word_part_syntax_text(first);
        let last_syntax = self.word_part_syntax_text(last);

        if !first_syntax.starts_with('(') || !last_syntax.ends_with(')') {
            return None;
        }

        let inner_start = word.span.start.advanced_by("(");
        if self.word_syntax_is_source_backed(word) {
            let syntax = word.span.slice(self.input);
            return Some((
                Cow::Borrowed(&syntax[1..syntax.len().saturating_sub(1)]),
                inner_start,
            ));
        }

        let mut inner = String::new();
        for (index, part) in word.parts.iter().enumerate() {
            let syntax = self.word_part_syntax_text(part);
            let start = if index == 0 { 1 } else { 0 };
            let end = syntax.len() - usize::from(index + 1 == word.parts.len());
            if start < end {
                inner.push_str(&syntax[start..end]);
            }
        }

        Some((Cow::Owned(inner), inner_start))
    }

    pub(in crate::parser) fn push_word_part_syntax(
        &self,
        out: &mut String,
        part: &WordPart,
        span: Span,
    ) {
        if self.word_part_syntax_is_source_backed(part, span) {
            out.push_str(span.slice(self.input));
            return;
        }

        match part {
            WordPart::Literal(text) => out.push_str(text.as_str(self.input, span)),
            WordPart::SingleQuoted { value, dollar } => {
                if *dollar {
                    out.push('$');
                }
                out.push('\'');
                out.push_str(value.slice(self.input));
                out.push('\'');
            }
            WordPart::DoubleQuoted { parts, dollar } => {
                if *dollar {
                    out.push('$');
                }
                out.push('"');
                for part in parts {
                    self.push_word_part_syntax(out, &part.kind, part.span);
                }
                out.push('"');
            }
            WordPart::Variable(name) => {
                out.push('$');
                out.push_str(name.as_str());
            }
            WordPart::CommandSubstitution { syntax, .. } => match syntax {
                CommandSubstitutionSyntax::DollarParen => out.push_str("$()"),
                CommandSubstitutionSyntax::Backtick => out.push_str("``"),
            },
            WordPart::ArithmeticExpansion {
                expression, syntax, ..
            } => match syntax {
                ArithmeticExpansionSyntax::DollarParenParen => {
                    out.push_str("$((");
                    out.push_str(expression.slice(self.input));
                    out.push_str("))");
                }
                ArithmeticExpansionSyntax::LegacyBracket => {
                    out.push_str("$[");
                    out.push_str(expression.slice(self.input));
                    out.push(']');
                }
            },
            WordPart::Parameter(parameter) => {
                out.push_str("${");
                out.push_str(parameter.raw_body.slice(self.input));
                out.push('}');
            }
            WordPart::ParameterExpansion {
                reference,
                operator,
                operand,
                colon_variant,
                ..
            } => {
                out.push_str("${");
                self.push_var_ref_syntax(out, reference);
                self.push_parameter_operator_syntax(
                    out,
                    operator,
                    operand.as_ref().map(|v| &**v),
                    *colon_variant,
                );
                out.push('}');
            }
            WordPart::Length(reference) => {
                out.push_str("${#");
                self.push_var_ref_syntax(out, reference);
                out.push('}');
            }
            WordPart::ArrayAccess(reference) => {
                out.push_str("${");
                self.push_var_ref_syntax(out, reference);
                out.push('}');
            }
            WordPart::ArrayLength(reference) => {
                out.push_str("${#");
                self.push_var_ref_syntax(out, reference);
                out.push('}');
            }
            WordPart::ArrayIndices(reference) => {
                out.push_str("${!");
                self.push_var_ref_syntax(out, reference);
                out.push('}');
            }
            WordPart::Substring {
                reference,
                offset,
                length,
                ..
            }
            | WordPart::ArraySlice {
                reference,
                offset,
                length,
                ..
            } => {
                out.push_str("${");
                self.push_var_ref_syntax(out, reference);
                out.push(':');
                out.push_str(offset.slice(self.input));
                if let Some(length) = length {
                    out.push(':');
                    out.push_str(length.slice(self.input));
                }
                out.push('}');
            }
            WordPart::IndirectExpansion {
                reference,
                operator,
                operand,
                colon_variant,
                ..
            } => {
                out.push_str("${!");
                self.push_var_ref_syntax(out, reference);
                if let Some(operator) = operator {
                    self.push_parameter_operator_syntax(
                        out,
                        operator,
                        operand.as_ref().map(|v| &**v),
                        *colon_variant,
                    );
                }
                out.push('}');
            }
            WordPart::PrefixMatch { prefix, kind } => {
                out.push_str("${!");
                out.push_str(prefix.as_str());
                out.push(kind.as_char());
                out.push('}');
            }
            WordPart::ProcessSubstitution { is_input, .. } => {
                out.push(if *is_input { '<' } else { '>' });
                out.push_str("()");
            }
            WordPart::Transformation {
                reference,
                operator,
            } => {
                out.push_str("${");
                self.push_var_ref_syntax(out, reference);
                out.push('@');
                out.push(*operator);
                out.push('}');
            }
        }
    }

    pub(in crate::parser) fn push_var_ref_syntax(&self, out: &mut String, reference: &VarRef) {
        out.push_str(reference.name.as_str());
        if let Some(subscript) = &reference.subscript {
            out.push('[');
            out.push_str(subscript.syntax_text(self.input));
            out.push(']');
        }
    }

    pub(in crate::parser) fn push_parameter_operator_syntax(
        &self,
        out: &mut String,
        operator: &ParameterOp,
        operand: Option<&SourceText>,
        colon_variant: bool,
    ) {
        let colon = if colon_variant { ":" } else { "" };
        match operator {
            ParameterOp::UseDefault => {
                out.push_str(colon);
                out.push('-');
                if let Some(operand) = operand {
                    out.push_str(operand.slice(self.input));
                }
            }
            ParameterOp::AssignDefault => {
                out.push_str(colon);
                out.push('=');
                if let Some(operand) = operand {
                    out.push_str(operand.slice(self.input));
                }
            }
            ParameterOp::UseReplacement => {
                out.push_str(colon);
                out.push('+');
                if let Some(operand) = operand {
                    out.push_str(operand.slice(self.input));
                }
            }
            ParameterOp::Error => {
                out.push_str(colon);
                out.push('?');
                if let Some(operand) = operand {
                    out.push_str(operand.slice(self.input));
                }
            }
            ParameterOp::RemovePrefixShort { pattern } => {
                out.push('#');
                self.push_pattern_syntax(out, pattern);
            }
            ParameterOp::RemovePrefixLong { pattern } => {
                out.push_str("##");
                self.push_pattern_syntax(out, pattern);
            }
            ParameterOp::RemoveSuffixShort { pattern } => {
                out.push('%');
                self.push_pattern_syntax(out, pattern);
            }
            ParameterOp::RemoveSuffixLong { pattern } => {
                out.push_str("%%");
                self.push_pattern_syntax(out, pattern);
            }
            ParameterOp::ReplaceFirst {
                pattern,
                replacement,
                ..
            } => {
                out.push('/');
                self.push_pattern_syntax(out, pattern);
                out.push('/');
                out.push_str(replacement.slice(self.input));
            }
            ParameterOp::ReplaceAll {
                pattern,
                replacement,
                ..
            } => {
                out.push_str("//");
                self.push_pattern_syntax(out, pattern);
                out.push('/');
                out.push_str(replacement.slice(self.input));
            }
            ParameterOp::UpperFirst => out.push('^'),
            ParameterOp::UpperAll => out.push_str("^^"),
            ParameterOp::LowerFirst => out.push(','),
            ParameterOp::LowerAll => out.push(','),
        }
    }

    pub(in crate::parser) fn push_pattern_syntax(&self, out: &mut String, pattern: &Pattern) {
        if pattern.is_source_backed() && pattern.span.end.offset() <= self.input.len() {
            out.push_str(pattern.span.slice(self.input));
            return;
        }

        for part in &pattern.parts {
            self.push_pattern_part_syntax(out, &part.kind, part.span);
        }
    }

    pub(in crate::parser) fn push_pattern_part_syntax(
        &self,
        out: &mut String,
        part: &PatternPart,
        span: Span,
    ) {
        match part {
            PatternPart::Literal(text) => out.push_str(text.as_str(self.input, span)),
            PatternPart::AnyString => out.push('*'),
            PatternPart::AnyChar => out.push('?'),
            PatternPart::CharClass(text) => out.push_str(text.slice(self.input)),
            PatternPart::Group { kind, patterns } => {
                out.push(kind.prefix());
                out.push('(');
                for (index, pattern) in patterns.iter().enumerate() {
                    if index > 0 {
                        out.push('|');
                    }
                    self.push_pattern_syntax(out, pattern);
                }
                out.push(')');
            }
            PatternPart::Word(word) => {
                for part in &word.parts {
                    self.push_word_part_syntax(out, &part.kind, part.span);
                }
            }
        }
    }
}
