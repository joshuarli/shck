use super::*;

#[test]
fn test_case_patterns_consume_segmented_tokens_directly() {
    let input = "case $x in foo\"bar\"|'baz'qux) echo hi ;; esac";
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Case(command) = compound else {
        panic!("expected case command");
    };

    let patterns = &command.cases[0].patterns;
    assert_eq!(patterns.len(), 2);

    assert_eq!(patterns[0].render(input), "foobar");
    assert_eq!(patterns[0].parts.len(), 2);
    assert_eq!(
        pattern_part_slices(&patterns[0], input),
        vec!["foo", "\"bar\""]
    );
    assert!(matches!(
        &patterns[0].parts[1].kind,
        PatternPart::Word(word) if is_fully_quoted(word)
    ));

    assert_eq!(patterns[1].render(input), "bazqux");
    assert_eq!(patterns[1].parts.len(), 2);
    assert_eq!(
        pattern_part_slices(&patterns[1], input),
        vec!["'baz'", "qux"]
    );
    assert!(matches!(
        &patterns[1].parts[0].kind,
        PatternPart::Word(word) if is_fully_quoted(word)
    ));
}

#[test]
fn test_case_accepts_literal_left_brace_pattern() {
    let input = concat!(
        "case \"$word\" in\n",
        "  \\(\\)) : ;;\n",
        "  {) : ;;\n",
        "  :) : ;;\n",
        "esac\n",
    );
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Case(command) = compound else {
        panic!("expected case command");
    };

    assert_eq!(command.cases.len(), 3);
    assert_eq!(command.cases[0].patterns[0].render_syntax(input), "\\(\\)");
    assert_eq!(command.cases[1].patterns[0].render_syntax(input), "{");
    assert_eq!(command.cases[2].patterns[0].render_syntax(input), ":");
}

#[test]
fn test_case_preserves_bash_fallthrough_terminator_spans() {
    let input = concat!(
        "case $mode in\n",
        "  start) printf '%s\\n' start ;&\n",
        "  stop) printf '%s\\n' stop ;;&\n",
        "esac\n",
    );
    let script = Parser::new(input).parse().unwrap().file;

    let (compound, _) = expect_compound(&script.body[0]);
    let AstCompoundCommand::Case(command) = compound else {
        panic!("expected case command");
    };

    assert_eq!(command.cases[0].terminator, CaseTerminator::FallThrough);
    assert_eq!(command.cases[0].terminator_span.unwrap().slice(input), ";&");
    assert_eq!(command.cases[1].terminator, CaseTerminator::Continue);
    assert_eq!(
        command.cases[1].terminator_span.unwrap().slice(input),
        ";;&"
    );
}

