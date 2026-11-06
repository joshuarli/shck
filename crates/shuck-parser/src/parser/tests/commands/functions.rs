use super::*;

#[test]
fn test_posix_function_with_brace_group_preserves_surface_form() {
    let input = "inc() { :; }\n";
    let script = Parser::new(input).parse().unwrap().file;

    let function = expect_function(&script.body[0]);
    let (compound, redirects) = expect_compound(function.body.as_ref());

    assert!(!function.uses_function_keyword());
    assert!(function.has_name_parens());
    assert_eq!(function.header.function_keyword_span, None);
    assert_eq!(
        function
            .header
            .trailing_parens_span
            .map(|span| span.slice(input)),
        Some("()")
    );
    assert!(matches!(compound, AstCompoundCommand::BraceGroup(_)));
    assert!(redirects.is_empty());
}

#[test]
fn test_alias_expansion_does_not_replace_posix_function_name() {
    let input = "\
shopt -s expand_aliases
alias wget='wget -V'
wget() { echo hi; }
";
    let script = Parser::new(input).parse().unwrap().file;

    let function = expect_function(&script.body[2]);

    assert_eq!(
        function.static_names().next().map(|name| name.as_str()),
        Some("wget")
    );
}

#[test]
fn test_posix_function_allows_subshell_body() {
    let input = "inc_subshell() ( j=$((j+5)); )\n";
    let script = Parser::new(input).parse().unwrap().file;

    let function = expect_function(&script.body[0]);
    let (compound, redirects) = expect_compound(function.body.as_ref());
    let AstCompoundCommand::Subshell(body) = compound else {
        panic!("expected subshell function body");
    };
    assert!(!function.uses_function_keyword());
    assert!(function.has_name_parens());
    assert!(redirects.is_empty());
    assert_eq!(body.len(), 1);
}

#[test]
fn test_posix_function_allows_arithmetic_body() {
    let input = "f() (( x + 1 ))\n";
    let script = Parser::new(input).parse().unwrap().file;

    let function = expect_function(&script.body[0]);
    let (compound, redirects) = expect_compound(function.body.as_ref());
    let AstCompoundCommand::Arithmetic(command) = compound else {
        panic!("expected arithmetic function body");
    };

    assert!(!function.uses_function_keyword());
    assert!(function.has_name_parens());
    assert!(redirects.is_empty());
    assert_eq!(command.span.slice(input), "(( x + 1 ))");
}

#[test]
fn test_posix_function_allows_if_body() {
    let input = "f() if true; then :; fi\n";
    let script = Parser::new(input).parse().unwrap().file;

    let function = expect_function(&script.body[0]);
    let (compound, redirects) = expect_compound(function.body.as_ref());
    assert!(matches!(compound, AstCompoundCommand::If(_)));

    assert!(!function.uses_function_keyword());
    assert!(function.has_name_parens());
    assert!(redirects.is_empty());
}

#[test]
fn test_function_keyword_allows_plus_name_in_bash() {
    let input = "function run++ () {\n  ((run_count+=1))\n}\nrun++\n";
    let parsed = Parser::new(input).parse().unwrap();

    assert_eq!(parsed.file.body.len(), 2);
    let function = expect_function(&parsed.file.body[0]);
    assert_eq!(
        function.static_names().next().map(|name| name.as_str()),
        Some("run++")
    );
}

#[test]
fn test_function_body_rejects_simple_command() {
    let parser = Parser::new("f() echo hi\n");
    assert!(
        parser.parse().is_err(),
        "simple command should not be accepted as a function body"
    );
}

#[test]
fn test_function_body_rejects_time_command() {
    let parser = Parser::new("f() time { :; }\n");
    assert!(
        parser.parse().is_err(),
        "time command should not be accepted as a function body"
    );
}

#[test]
fn test_function_body_rejects_coproc_command() {
    let parser = Parser::new("f() coproc cat\n");
    assert!(
        parser.parse().is_err(),
        "coproc command should not be accepted as a function body"
    );
}

#[test]
fn test_empty_function_body_rejected() {
    let parser = Parser::new("f() { }");
    assert!(
        parser.parse().is_err(),
        "empty function body should be rejected"
    );
}

#[test]
fn test_nonempty_function_body_accepted() {
    let parser = Parser::new("f() { echo hi; }");
    assert!(
        parser.parse().is_ok(),
        "non-empty function body should be accepted"
    );
}

#[test]
fn test_parse_fixture_style_function_after_awk_dollar_program() {
    let input = r#"
parse_file_col_to_csv(){
    local field="$2"
    awk "{print \$$field}" "$data_file" |
    tr '\n' ',' |
    sed 's/,/, /g; s/, $//'
}

file_modified_in_last_days(){
    local file="$1"
    local days="$2"
    if ! is_int "$days"; then
        die "bad days"
    fi
    if ! [ -f "$file" ]; then
        return 1
    elif find "$file" -mtime -"$days" -print | grep -q .; then
        return 0
    else
        local days_ago_in_seconds
        days_ago_in_seconds="$(date -d "$days days ago" '+%s')"
        if is_mac; then
            if [ "$(stat -f '%m' "$file")" -ge "$days_ago_in_seconds" ]; then
                return 0
            else
                return 1
            fi
        elif [ "$(stat -c '%Y' "$file")" -ge "$days_ago_in_seconds" ]; then
            return 0
        else
            return 1
        fi
    fi
    }
"#;
    let parsed = Parser::with_dialect(input, ShellDialect::Bash).parse();
    assert_eq!(
        parsed.status,
        ParseStatus::Clean,
        "{}",
        parsed.strict_error()
    );
}

#[test]
fn test_parse_arithmetic_for_with_line_continuations_inside_function_body() {
    let input = "\
subcommand()
{
    local args_index=$positional_start
    local usage_args_index
    for ((\\
    usage_args_index = 0;  \\
    usage_args_index < ${#args[@]};  \\
    args_index++, usage_args_index++)); do
        echo ok
    done
}
";
    let parsed = Parser::new(input).parse();

    assert_eq!(parsed.status, ParseStatus::Clean);
    assert!(parsed.diagnostics.is_empty());
    assert!(parsed.terminal_error.is_none());

    let function = expect_function(&parsed.file.body[0]);
    let (compound, redirects) = expect_compound(function.body.as_ref());
    let AstCompoundCommand::BraceGroup(body) = compound else {
        panic!("expected brace-group function body");
    };

    assert!(redirects.is_empty());
    assert_eq!(body.len(), 3);
    let (compound, redirects) = expect_compound(&body[2]);
    let AstCompoundCommand::ArithmeticFor(command) = compound else {
        panic!("expected arithmetic for compound command");
    };
    assert!(redirects.is_empty());
    assert_eq!(
        command.init_span.unwrap().slice(input),
        "\\\n    usage_args_index = 0"
    );
    assert_eq!(
        command.condition_span.unwrap().slice(input),
        "  \\\n    usage_args_index < ${#args[@]}"
    );
    assert_eq!(
        command.step_span.unwrap().slice(input),
        "  \\\n    args_index++, usage_args_index++"
    );
}

