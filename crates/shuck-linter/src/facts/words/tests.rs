#[cfg(test)]
mod word_classification_tests {
    use super::*;

    fn parse_commands(source: &str) -> StmtSeq {
        Parser::new(source).parse().unwrap().file.body
    }

    #[test]
    fn quoting_literal_scan_preserves_backslash_runs_and_unicode_whitespace() {
        fn reference(text: &str, literal_newlines: bool) -> bool {
            if text.contains(['"', '\'']) { return true; }
            let mut chars = text.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch != '\\' { continue; }
                while chars.peek() == Some(&'\\') { chars.next(); }
                if chars.peek().is_some_and(|next| next.is_whitespace()
                    && (literal_newlines || !matches!(next, '\n' | '\r'))) { return true; }
            }
            false
        }
        let atoms = ["a", "é", "\\", "\n", "\r", " ", "\t", "\u{a0}", "\"", "'"];
        for a in atoms { for b in atoms { for c in atoms {
            let text = format!("{a}{b}{c}");
            for literal_newlines in [false, true] {
                let context = if literal_newlines {
                    ShellQuotingLiteralTextContext::LiteralBackslashNewlines
                } else { ShellQuotingLiteralTextContext::ShellContinuationAware };
                assert_eq!(text_contains_shell_quoting_literals(&text, context),
                    reference(&text, literal_newlines), "{text:?}");
            }
        } } }
    }

    #[test]
    fn reused_expansion_buffers_do_not_leak_between_words() {
        let source = r#"printf "$first" literal $(date) "${array[@]}" '$quoted' $last {a,b} end"#;
        let commands = parse_commands(source);
        let Command::Simple(command) = &commands[0].command else { panic!("simple command"); };
        let lines = shuck_indexer::LineIndex::new(source);
        let locator = Locator::new(source, &lines);
        let mut reused = DerivedWordTraversalSpans::default();
        for word in &command.args {
            let mut actual = ListArena::new();
            let mut expected = ListArena::new();
            let mut scratch = Vec::new();
            let (actual_data, actual_array) = derive_word_fact_data(word, locator,
                shuck_semantic::ShellDialect::Bash, &mut actual, &mut scratch, &mut reused, None);
            let (expected_data, expected_array) = derive_word_fact_data(word, locator,
                shuck_semantic::ShellDialect::Bash, &mut expected, &mut scratch,
                &mut DerivedWordTraversalSpans::default(), None);
            assert_eq!(actual.as_slice(), expected.as_slice());
            assert_eq!(format!("{actual_data:?}"), format!("{expected_data:?}"));
            assert_eq!(actual_array, expected_array);
        }
    }

    #[test]
    fn detects_alias_positional_parameters_with_runtime_quote_state() {
        assert!(contains_positional_parameter_reference("echo $1"));
        assert!(contains_positional_parameter_reference("echo \"${1}\""));
        assert!(contains_positional_parameter_reference("echo ${#1}"));
        assert!(contains_positional_parameter_reference("echo ${!1}"));
        assert!(contains_positional_parameter_reference(r"echo \$$1"));
        assert!(contains_positional_parameter_reference(r"echo \'$1"));
        assert!(contains_positional_parameter_reference("echo hi# $1"));
        assert!(!contains_positional_parameter_reference(r"echo \$1"));
        assert!(!contains_positional_parameter_reference(r"echo \${1}"));
        assert!(!contains_positional_parameter_reference("echo '$1'"));
        assert!(!contains_positional_parameter_reference("echo hi # $1"));
        assert!(!contains_positional_parameter_reference("echo hi; # $1"));
        assert!(!contains_positional_parameter_reference("echo hi;# $1"));
        assert!(!contains_positional_parameter_reference("echo hi &&# $1"));
        assert!(!contains_positional_parameter_reference("echo $$1"));
    }

    #[test]
    fn classify_word_distinguishes_fixed_literals_and_quoted_expansions() {
        let source = "printf \"literal\" \"prefix$foo\"\n";
        let commands = parse_commands(source);
        let Command::Simple(command) = &commands[0].command else {
            panic!("expected simple command");
        };

        let literal = classify_word(&command.args[0], source);
        assert_eq!(literal.quote, WordQuote::FullyQuoted);
        assert_eq!(literal.literalness, WordLiteralness::FixedLiteral);
        assert_eq!(literal.expansion_kind, WordExpansionKind::None);
        assert_eq!(literal.substitution_shape, WordSubstitutionShape::None);

        let expanded = classify_word(&command.args[1], source);
        assert_eq!(expanded.quote, WordQuote::FullyQuoted);
        assert_eq!(expanded.literalness, WordLiteralness::Expanded);
        assert_eq!(expanded.expansion_kind, WordExpansionKind::Scalar);
        assert_eq!(expanded.substitution_shape, WordSubstitutionShape::None);
    }

    #[test]
    fn classify_word_reports_plain_and_mixed_command_substitutions() {
        let source = "printf \"$(date)\" \"prefix$(date)\"\n";
        let commands = parse_commands(source);
        let Command::Simple(command) = &commands[0].command else {
            panic!("expected simple command");
        };

        assert_eq!(
            classify_word(&command.args[0], source).substitution_shape,
            WordSubstitutionShape::Plain
        );
        assert_eq!(
            classify_word(&command.args[1], source).substitution_shape,
            WordSubstitutionShape::Mixed
        );
    }

    #[test]
    fn classify_word_treats_escaped_backslash_before_command_substitution_as_mixed() {
        let source = "printf \"\\\\$(printf '%03o' \"$i\")\"\n";
        let commands = parse_commands(source);
        let Command::Simple(command) = &commands[0].command else {
            panic!("expected simple command");
        };

        let classification = classify_word(&command.args[0], source);
        assert_eq!(classification.quote, WordQuote::FullyQuoted);
        assert_eq!(classification.literalness, WordLiteralness::Expanded);
        assert_eq!(
            classification.substitution_shape,
            WordSubstitutionShape::Mixed
        );
    }

    #[test]
    fn classify_word_reports_scalar_and_array_expansions() {
        let source = "printf $foo ${arr[@]} ${arr[0]} ${arr[@]:1}\n";
        let commands = parse_commands(source);
        let Command::Simple(command) = &commands[0].command else {
            panic!("expected simple command");
        };

        assert_eq!(
            classify_word(&command.args[0], source).expansion_kind,
            WordExpansionKind::Scalar
        );
        assert_eq!(
            classify_word(&command.args[1], source).expansion_kind,
            WordExpansionKind::Array
        );
        assert_eq!(
            classify_word(&command.args[2], source).expansion_kind,
            WordExpansionKind::Scalar
        );
        assert_eq!(
            classify_word(&command.args[3], source).expansion_kind,
            WordExpansionKind::Array
        );
    }

    #[test]
    fn plain_parameter_reference_accepts_single_direct_expansions_only() {
        let source = "\
printf '%s\\n' \
$name \"$name\" ${name} \"${name}\" $1 \"$#\" \"$@\" ${*} \
${@:2} ${arr[0]} ${arr[@]} ${!name} ${name:-fallback} \"$@$@\" \"prefix$name\"\n";
        let commands = parse_commands(source);
        let Command::Simple(command) = &commands[0].command else {
            panic!("expected simple command");
        };

        let plain = command
            .args
            .iter()
            .skip(1)
            .map(word_is_plain_parameter_reference)
            .collect::<Vec<_>>();

        assert_eq!(
            plain,
            vec![
                true, true, true, true, true, true, true, true, false, false, false, false, false,
                false, false
            ]
        );
    }

    #[test]
    fn classify_test_and_conditional_operands_share_literal_runtime_decisions() {
        let source = "test foo\ntest ~\n[[ \"$re\" ]]\n[[ literal ]]\n[[ ~ ]]\n";
        let commands = parse_commands(source);

        let Command::Simple(simple_test) = &commands[0].command else {
            panic!("expected simple command");
        };
        assert_eq!(
            classify_contextual_operand(
                &simple_test.args[0],
                source,
                ExpansionContext::CommandArgument
            ),
            TestOperandClass::FixedLiteral
        );

        let Command::Simple(runtime_test) = &commands[1].command else {
            panic!("expected simple command");
        };
        assert_eq!(
            classify_contextual_operand(
                &runtime_test.args[0],
                source,
                ExpansionContext::CommandArgument
            ),
            TestOperandClass::RuntimeSensitive
        );

        let Command::Compound(CompoundCommand::Conditional(runtime)) = &commands[2].command else {
            panic!("expected conditional");
        };
        assert_eq!(
            classify_conditional_operand(&runtime.expression, source),
            TestOperandClass::RuntimeSensitive
        );

        let Command::Compound(CompoundCommand::Conditional(literal)) = &commands[3].command else {
            panic!("expected conditional");
        };
        assert_eq!(
            classify_conditional_operand(&literal.expression, source),
            TestOperandClass::FixedLiteral
        );

        let Command::Compound(CompoundCommand::Conditional(runtime)) = &commands[4].command else {
            panic!("expected conditional");
        };
        assert_eq!(
            classify_conditional_operand(&runtime.expression, source),
            TestOperandClass::RuntimeSensitive
        );
    }

    #[test]
    fn contextual_operand_classification_respects_regex_and_case_contexts() {
        let source = "printf ~ *.sh {a,b}\n";
        let commands = parse_commands(source);
        let Command::Simple(command) = &commands[0].command else {
            panic!("expected simple command");
        };

        assert_eq!(
            classify_contextual_operand(&command.args[0], source, ExpansionContext::RegexOperand),
            TestOperandClass::RuntimeSensitive
        );
        assert_eq!(
            classify_contextual_operand(&command.args[1], source, ExpansionContext::CasePattern),
            TestOperandClass::FixedLiteral
        );
        assert_eq!(
            classify_contextual_operand(&command.args[2], source, ExpansionContext::CasePattern),
            TestOperandClass::FixedLiteral
        );
    }
}

#[cfg(test)]
mod expansion_analysis_tests {
    use shuck_ast::Command;
    use shuck_parser::parser::{Parser, ShellDialect};

    use super::{
        ComparablePathKey, ComparablePathPart, ExpansionAnalysis, ExpansionContext,
        ExpansionValueShape, RedirectDevNullStatus, WordLiteralness, WordQuote,
        analyze_literal_runtime, analyze_redirect_target, analyze_word, comparable_path,
    };
    use crate::{
        FieldSplittingBehavior, GlobDotBehavior, GlobFailureBehavior, PathnameExpansionBehavior,
        PatternOperatorBehavior,
    };

    fn parse_argument_words(source: &str) -> Vec<shuck_ast::Word> {
        let file = Parser::new(source).parse().unwrap().file;
        let Command::Simple(command) = &file.body[0].command else {
            panic!("expected simple command");
        };
        command.args.to_vec()
    }

    fn analyze_argument_words(source: &str) -> Vec<ExpansionAnalysis> {
        parse_argument_words(source)
            .iter()
            .map(|word| analyze_word(word, source, None))
            .collect()
    }

    fn analyze_argument_words_with_dialect(
        source: &str,
        dialect: ShellDialect,
    ) -> Vec<ExpansionAnalysis> {
        let file = Parser::with_dialect(source, dialect).parse().unwrap().file;
        let Command::Simple(command) = &file.body[0].command else {
            panic!("expected simple command");
        };
        command
            .args
            .iter()
            .map(|word| analyze_word(word, source, None))
            .collect()
    }

    #[test]
    fn analyze_word_tracks_array_values_and_multi_field_expansions_separately() {
        let analyses = analyze_argument_words(
            "printf %s ${arr[@]} \"${arr[*]}\" ${!prefix@} ${!name} ${value@Q}\n",
        );

        assert_eq!(analyses[1].value_shape, ExpansionValueShape::MultiField);
        assert!(analyses[1].array_valued);
        assert!(analyses[1].can_expand_to_multiple_fields);

        assert_eq!(analyses[2].quote, WordQuote::FullyQuoted);
        assert_eq!(analyses[2].value_shape, ExpansionValueShape::Array);
        assert!(analyses[2].array_valued);
        assert!(!analyses[2].can_expand_to_multiple_fields);

        assert_eq!(analyses[3].value_shape, ExpansionValueShape::MultiField);
        assert!(!analyses[3].array_valued);
        assert!(analyses[3].can_expand_to_multiple_fields);

        assert_eq!(analyses[4].value_shape, ExpansionValueShape::Unknown);
        assert_eq!(analyses[4].literalness, WordLiteralness::Expanded);

        assert_eq!(analyses[5].value_shape, ExpansionValueShape::MultiField);
        assert_eq!(analyses[5].literalness, WordLiteralness::Expanded);
        assert!(!analyses[5].array_valued);
    }

    #[test]
    fn analyze_word_marks_prefix_match_at_as_multi_field_even_when_quoted() {
        let analyses = analyze_argument_words("printf %s \"${!prefix@}\" \"${!prefix*}\"\n");

        assert_eq!(analyses[1].value_shape, ExpansionValueShape::MultiField);
        assert!(analyses[1].can_expand_to_multiple_fields);

        assert_eq!(analyses[2].value_shape, ExpansionValueShape::Unknown);
        assert!(!analyses[2].can_expand_to_multiple_fields);
    }

    #[test]
    fn analyze_word_treats_bourne_transformations_as_split_and_glob_hazards() {
        let analyses = analyze_argument_words("printf %s ${name@U}\n");

        assert_eq!(analyses[1].value_shape, ExpansionValueShape::MultiField);
        assert!(analyses[1].hazards.field_splitting);
        assert!(analyses[1].hazards.pathname_matching);
        assert!(analyses[1].can_expand_to_multiple_fields);
    }

    #[test]
    fn analyze_literal_runtime_tracks_globs_in_mixed_words() {
        let source =
            "printf '%s\\n' \"$basedir/\"* \"$(dirname \"$0\")\"/../docs/usage/distrobox*\n";
        let words = parse_argument_words(source);
        let first = analyze_literal_runtime(&words[1], source, ExpansionContext::ForList, None);
        let second = analyze_literal_runtime(&words[2], source, ExpansionContext::ForList, None);

        assert!(first.hazards.pathname_matching);
        assert!(first.is_runtime_sensitive());
        assert!(second.hazards.pathname_matching);
        assert!(second.is_runtime_sensitive());
    }

    #[test]
    fn analyze_redirect_target_distinguishes_descriptor_dups_and_dev_null() {
        let static_dup_source = "echo hi 2>&3\n";
        let static_dup_file = Parser::new(static_dup_source).parse().unwrap().file;
        let Command::Simple(_) = &static_dup_file.body[0].command else {
            panic!("expected simple command");
        };
        let static_dup = analyze_redirect_target(
            &static_dup_file.body[0].redirects[0],
            static_dup_source,
            None,
        )
        .expect("expected redirect analysis");
        assert!(static_dup.is_descriptor_dup());
        assert_eq!(static_dup.numeric_descriptor_target, Some(3));
        assert!(!static_dup.is_runtime_sensitive());

        let file_source = "echo hi > /dev/null\n";
        let file_commands = Parser::new(file_source).parse().unwrap().file;
        let Command::Simple(_) = &file_commands.body[0].command else {
            panic!("expected simple command");
        };
        let file = analyze_redirect_target(&file_commands.body[0].redirects[0], file_source, None)
            .expect("expected redirect analysis");
        assert!(file.is_file_target());
        assert!(file.is_definitely_dev_null());
        assert!(!file.is_runtime_sensitive());

        let maybe_source = "echo hi > \"$target\"\n";
        let maybe_commands = Parser::new(maybe_source).parse().unwrap().file;
        let Command::Simple(_) = &maybe_commands.body[0].command else {
            panic!("expected simple command");
        };
        let maybe =
            analyze_redirect_target(&maybe_commands.body[0].redirects[0], maybe_source, None)
                .expect("expected redirect analysis");
        assert!(maybe.is_file_target());
        assert_eq!(
            maybe.dev_null_status,
            Some(RedirectDevNullStatus::MaybeDevNull)
        );
        assert!(maybe.is_runtime_sensitive());

        let fanout_source = "echo hi > ${targets[@]}\n";
        let fanout_commands = Parser::new(fanout_source).parse().unwrap().file;
        let Command::Simple(_) = &fanout_commands.body[0].command else {
            panic!("expected simple command");
        };
        let fanout =
            analyze_redirect_target(&fanout_commands.body[0].redirects[0], fanout_source, None)
                .expect("expected redirect analysis");
        assert!(fanout.can_expand_to_multiple_fields());
        assert!(fanout.is_runtime_sensitive());

        let tilde_source = "echo hi > ~/*.log\n";
        let tilde_commands = Parser::new(tilde_source).parse().unwrap().file;
        let Command::Simple(_) = &tilde_commands.body[0].command else {
            panic!("expected simple command");
        };
        let tilde =
            analyze_redirect_target(&tilde_commands.body[0].redirects[0], tilde_source, None)
                .expect("expected redirect analysis");
        assert!(tilde.is_file_target());
        assert_eq!(
            tilde.dev_null_status,
            Some(RedirectDevNullStatus::MaybeDevNull)
        );
        assert!(tilde.runtime_literal.is_runtime_sensitive());
        assert!(tilde.is_runtime_sensitive());
    }

    #[test]
    fn analyze_literal_runtime_tracks_context_sensitive_literals() {
        let source = "printf ~ ~user x=~ *.sh {a,b} \"~\" '*.sh' \"{a,b}\"\n";
        let words = parse_argument_words(source);

        assert!(
            analyze_literal_runtime(&words[0], source, ExpansionContext::CommandArgument, None)
                .hazards
                .tilde_expansion
        );
        assert!(
            analyze_literal_runtime(&words[1], source, ExpansionContext::CommandArgument, None)
                .hazards
                .tilde_expansion
        );
        assert!(
            analyze_literal_runtime(&words[2], source, ExpansionContext::CommandArgument, None)
                .hazards
                .tilde_expansion
        );
        assert!(
            analyze_literal_runtime(&words[3], source, ExpansionContext::CommandArgument, None)
                .hazards
                .pathname_matching
        );
        assert!(
            analyze_literal_runtime(&words[4], source, ExpansionContext::CommandArgument, None)
                .hazards
                .brace_fanout
        );

        assert!(
            !analyze_literal_runtime(&words[5], source, ExpansionContext::CommandArgument, None)
                .is_runtime_sensitive()
        );
        assert!(
            !analyze_literal_runtime(&words[6], source, ExpansionContext::CommandArgument, None)
                .is_runtime_sensitive()
        );
        assert!(
            !analyze_literal_runtime(&words[7], source, ExpansionContext::CommandArgument, None)
                .is_runtime_sensitive()
        );

        assert!(
            analyze_literal_runtime(&words[0], source, ExpansionContext::StringTestOperand, None)
                .hazards
                .tilde_expansion
        );
        assert!(
            analyze_literal_runtime(&words[0], source, ExpansionContext::RegexOperand, None)
                .hazards
                .tilde_expansion
        );
        assert!(
            !analyze_literal_runtime(&words[3], source, ExpansionContext::StringTestOperand, None)
                .is_runtime_sensitive()
        );
        assert!(
            !analyze_literal_runtime(&words[4], source, ExpansionContext::CasePattern, None)
                .is_runtime_sensitive()
        );
    }

    #[test]
    fn analyze_literal_runtime_treats_loop_lists_like_argument_lists() {
        let source = "printf ~ *.sh {a,b}\n";
        let words = parse_argument_words(source);

        assert!(
            analyze_literal_runtime(&words[0], source, ExpansionContext::ForList, None)
                .hazards
                .tilde_expansion
        );
        assert!(
            analyze_literal_runtime(&words[1], source, ExpansionContext::ForList, None)
                .hazards
                .pathname_matching
        );
        assert!(
            analyze_literal_runtime(&words[2], source, ExpansionContext::ForList, None)
                .hazards
                .brace_fanout
        );
    }
}
