use super::*;

#[test]
fn test_posix_function_allows_conditional_body() {
    let input = "f() [[ -n \"$x\" ]]\n";
    let script = Parser::new(input).parse().unwrap().file;

    let function = expect_function(&script.body[0]);
    let (compound, redirects) = expect_compound(function.body.as_ref());
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional function body");
    };

    assert!(!function.uses_function_keyword());
    assert!(function.has_name_parens());
    assert!(redirects.is_empty());
    assert_eq!(command.span.slice(input), "[[ -n \"$x\" ]]");
}

#[test]
fn test_posix_function_parses_negated_bracket_if_body() {
    let input = "\
recent_file(){
    if ! [ -f \"$file\" ]; then
        return 1
    elif find \"$file\" -mtime -\"$days\" -print | grep -q .; then
        return 0
    else
        local days_ago_in_seconds
        days_ago_in_seconds=\"$(date -d \"$days days ago\" '+%s')\"
        if is_mac; then
            if [ \"$(stat -f '%m' \"$file\")\" -ge \"$days_ago_in_seconds\" ]; then
                return 0
            else
                return 1
            fi
        elif [ \"$(stat -c '%Y' \"$file\")\" -ge \"$days_ago_in_seconds\" ]; then
            return 0
        else
            return 1
        fi
    fi
}
";
    let parsed = Parser::new(input).parse().unwrap();

    assert!(matches!(
        expect_function(&parsed.file.body[0]).body.command,
        AstCommand::Compound(..)
    ));
}

#[test]
fn test_double_left_paren_test_clause_parses_as_command() {
    let input =
        "if ! ((test x\\\"$i\\\" = x-g) || (test x\\\"$i\\\" = x-O2)); then\n  echo bye\nfi\n";
    Parser::new(input).parse().unwrap();
}

#[test]
fn test_double_left_paren_pipeline_parses_as_command() {
    let input = "((cat </dev/zero; echo $? >&7) | true) 7>&1\n";
    Parser::new(input).parse().unwrap();
}

#[test]
fn test_parse_conditional_builds_structured_logical_ast() {
    let script = Parser::new("[[ ! (foo && bar) ]]\n").parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional compound command");
    };

    let ConditionalExpr::Unary(unary) = &command.expression else {
        panic!("expected unary conditional");
    };
    assert_eq!(unary.op, ConditionalUnaryOp::Not);

    let ConditionalExpr::Parenthesized(paren) = unary.expr.as_ref() else {
        panic!("expected parenthesized conditional");
    };
    let ConditionalExpr::Binary(binary) = paren.expr.as_ref() else {
        panic!("expected binary conditional");
    };
    assert_eq!(binary.op, ConditionalBinaryOp::And);
    assert!(matches!(binary.left.as_ref(), ConditionalExpr::Word(_)));
    assert!(matches!(binary.right.as_ref(), ConditionalExpr::Word(_)));
    assert_eq!(command.left_bracket_span.start.column(), 1);
    assert_eq!(command.right_bracket_span.start.column(), 19);
}

#[test]
fn test_parse_conditional_accepts_nested_grouping_with_double_parens() {
    let input = "[[ ! -e \"$cache\" && (( -e \"$prefix/n\" && ! -w \"$prefix/n\" ) || ( ! -e \"$prefix/n\" && ! -w \"$prefix\" )) ]]\n";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional compound command");
    };

    let ConditionalExpr::Binary(binary) = &command.expression else {
        panic!("expected binary conditional");
    };
    assert_eq!(binary.op, ConditionalBinaryOp::And);

    let ConditionalExpr::Parenthesized(paren) = binary.right.as_ref() else {
        panic!("expected parenthesized conditional term");
    };
    assert_eq!(
        paren.span().slice(input),
        "(( -e \"$prefix/n\" && ! -w \"$prefix/n\" ) || ( ! -e \"$prefix/n\" && ! -w \"$prefix\" ))"
    );

    let ConditionalExpr::Binary(inner) = paren.expr.as_ref() else {
        panic!("expected grouped binary conditional");
    };
    assert_eq!(inner.op, ConditionalBinaryOp::Or);
    assert!(matches!(
        inner.left.as_ref(),
        ConditionalExpr::Parenthesized(_)
    ));
    assert!(matches!(
        inner.right.as_ref(),
        ConditionalExpr::Parenthesized(_)
    ));
}

#[test]
fn test_parse_conditional_accepts_adjacent_group_closes_after_rhs_words() {
    let input = "[[ -n $brew_prefix && (($brew_prefix != \"/usr\" && $brew_prefix != \"/usr/local\") || (is_mac && osx_using_default_compiler && $CFLAGS =~ (^|\\ )-isysroot\\ )) ]]\n";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional compound command");
    };

    let ConditionalExpr::Binary(binary) = &command.expression else {
        panic!("expected binary conditional");
    };
    assert_eq!(binary.op, ConditionalBinaryOp::And);
    assert!(matches!(
        binary.right.as_ref(),
        ConditionalExpr::Parenthesized(_)
    ));
}

#[test]
fn test_parse_conditional_stops_quoted_rhs_before_adjacent_group_closes() {
    for input in ["[[ (( x == \"foo\" )) ]]\n", "[[ (( x =~ \"foo\" )) ]]\n"] {
        let script = Parser::new(input).parse().unwrap().file;

        let (compound, _) = expect_compound(&script.body[0]);
        let AstCompoundCommand::Conditional(command) = compound else {
            panic!("expected conditional compound command for {input}");
        };

        assert!(matches!(
            command.expression,
            ConditionalExpr::Parenthesized(_)
        ));
    }
}

#[test]
fn test_parse_conditional_pattern_rhs_preserves_structure() {
    let input = "[[ foo == @(bar|baz)* ]]\n";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional compound command");
    };

    let ConditionalExpr::Binary(binary) = &command.expression else {
        panic!("expected binary conditional");
    };
    assert_eq!(binary.op, ConditionalBinaryOp::PatternEq);

    let ConditionalExpr::Pattern(pattern) = binary.right.as_ref() else {
        panic!("expected pattern rhs");
    };
    assert_eq!(pattern.render(input), "@(bar|baz)*");
    assert!(matches!(
        &pattern.parts[0].kind,
        PatternPart::Group {
            kind: PatternGroupKind::ExactlyOne,
            ..
        }
    ));
    assert!(matches!(&pattern.parts[1].kind, PatternPart::AnyString));
}

#[test]
fn test_parse_conditional_var_ref_operand() {
    let input = "[[ -v assoc[$key] ]]\n";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional compound command");
    };

    let ConditionalExpr::Unary(unary) = &command.expression else {
        panic!("expected unary conditional");
    };
    assert_eq!(unary.op, ConditionalUnaryOp::VariableSet);

    let ConditionalExpr::VarRef(var_ref) = unary.expr.as_ref() else {
        panic!("expected typed var-ref operand");
    };
    assert_eq!(var_ref.name.as_str(), "assoc");
    assert_eq!(var_ref.name_span.slice(input), "assoc");
    expect_subscript(var_ref, input, "$key");
}

#[test]
fn test_parse_conditional_quoted_command_substitution_preserves_nested_quotes() {
    let input = "[[ \"$(get_permission \"$1\")\" != \"$(id -u)\" ]]\n";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional compound command");
    };

    let ConditionalExpr::Binary(binary) = &command.expression else {
        panic!("expected binary conditional");
    };
    assert_eq!(binary.op, ConditionalBinaryOp::PatternNe);

    let ConditionalExpr::Word(left) = binary.left.as_ref() else {
        panic!("expected left operand word");
    };
    let ConditionalExpr::Pattern(right) = binary.right.as_ref() else {
        panic!("expected right operand pattern");
    };
    assert_eq!(left.span.slice(input), "\"$(get_permission \"$1\")\"");
    assert_eq!(right.span.slice(input), "\"$(id -u)\"");

    let WordPart::DoubleQuoted { parts, .. } = &left.parts[0].kind else {
        panic!("expected double-quoted left operand");
    };
    let WordPart::CommandSubstitution { body, syntax } = &parts[0].kind else {
        panic!("expected left command substitution");
    };
    assert_eq!(*syntax, CommandSubstitutionSyntax::DollarParen);

    let inner = expect_simple(&body[0]);
    assert_eq!(inner.name.render(input), "get_permission");
    assert_eq!(inner.args[0].render_syntax(input), "\"$1\"");
}

#[test]
fn test_parse_conditional_regex_rhs_preserves_structure() {
    let input = "[[ foo =~ [ab](c|d) ]]\n";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional compound command");
    };

    let ConditionalExpr::Binary(binary) = &command.expression else {
        panic!("expected binary conditional");
    };
    assert_eq!(binary.op, ConditionalBinaryOp::RegexMatch);

    let ConditionalExpr::Regex(word) = binary.right.as_ref() else {
        panic!("expected regex rhs");
    };
    assert_eq!(word.render(input), "[ab](c|d)");
}

#[test]
fn test_parse_conditional_regex_rhs_with_double_left_paren_groups() {
    let input = "[[ x =~ ^\\\"\\-1[[:blank:]]((\\?[luds])+).* ]]\n";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional compound command");
    };

    let ConditionalExpr::Binary(binary) = &command.expression else {
        panic!("expected binary conditional");
    };
    assert_eq!(binary.op, ConditionalBinaryOp::RegexMatch);

    let ConditionalExpr::Regex(word) = binary.right.as_ref() else {
        panic!("expected regex rhs");
    };
    assert_eq!(word.render(input), "^\\\"\\-1[[:blank:]]((\\?[luds])+).*");
}

#[test]
fn test_parse_conditional_regex_rhs_keeps_parameter_pattern_with_literal_paren() {
    let input = "[[ \"${2}\" =~ ^${__theme%% (*} ]]\n";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional compound command");
    };

    let ConditionalExpr::Binary(binary) = &command.expression else {
        panic!("expected binary conditional");
    };
    assert_eq!(binary.op, ConditionalBinaryOp::RegexMatch);

    let ConditionalExpr::Regex(word) = binary.right.as_ref() else {
        panic!("expected regex rhs");
    };
    assert_eq!(word.render(input), "^${__theme%% (*}");
}

#[test]
fn test_parse_conditional_regex_allows_left_brace_operand() {
    let input = "[[ { =~ \"{\" ]]\n";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Conditional(command) = compound else {
        panic!("expected conditional compound command");
    };

    let ConditionalExpr::Binary(binary) = &command.expression else {
        panic!("expected binary conditional");
    };
    assert_eq!(binary.op, ConditionalBinaryOp::RegexMatch);

    let ConditionalExpr::Word(left) = binary.left.as_ref() else {
        panic!("expected literal left operand");
    };
    assert_eq!(left.span.slice(input), "{");

    let ConditionalExpr::Regex(right) = binary.right.as_ref() else {
        panic!("expected regex rhs");
    };
    assert_eq!(right.render(input), "{");
}

#[test]
fn test_parse_if_condition_accepts_chained_double_brackets_with_regex_brace_literal() {
    let input = "if [[ $MOTD ]] && ! [[ $MOTD =~ ^{ ]]; then\n  :\nfi\n";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, redirects) = expect_compound(&script.body[0]);
    let AstCompoundCommand::If(command) = compound else {
        panic!("expected if command");
    };

    assert!(redirects.is_empty());
    assert_eq!(command.condition.len(), 1);
    assert_eq!(expect_binary(&command.condition[0]).op, BinaryOp::And);
}

#[test]
fn test_posix_dialect_rejects_double_bracket_conditionals() {
    let error = Parser::with_dialect("[[ foo == bar ]]\n", ShellDialect::Posix)
        .parse()
        .unwrap_err();

    assert!(matches!(
        error,
        Error::Parse { message, .. } if message.contains("[[ ]] conditionals")
    ));
}

