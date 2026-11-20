use std::path::{Path, PathBuf};

use shuck_linter::{
    AnalysisRequest, Applicability, Diagnostic, FileContract, LinterSettings, PluginFramework, PluginRequest,
    PluginRequestKind, PluginResolution, PluginResolver, Rule, RuleSelector, RuleSet, Severity,
    SourcePathResolver, apply_fixes, apply_rule_selector_layer, parse_rule_selectors,
};
use shuck_parser::parser::{ParseResult, ParseStatus, Parser};

struct EmptyPluginResolver;

impl PluginResolver for EmptyPluginResolver {
    fn resolve_plugin_request(
        &self,
        _source_path: &Path,
        _request: &PluginRequest,
    ) -> PluginResolution {
        PluginResolution::default()
    }
}

fn accepts_source_path_resolver(_resolver: &(dyn SourcePathResolver + Send + Sync)) {}

fn inspect_parse_result(result: &ParseResult) -> (&shuck_ast::File, usize, ParseStatus) {
    (&result.file, result.diagnostics.len(), result.status)
}

fn inspect_diagnostic(diagnostic: &Diagnostic) -> (&str, Rule, Severity) {
    (&diagnostic.message, diagnostic.rule, diagnostic.severity)
}

fn is_selected_rule(rule: Rule) -> bool {
    matches!(rule, Rule::UnusedAssignment | Rule::UndefinedVariable)
}

#[test]
fn downstream_construction_and_inspection_path_stays_ergonomic() {
    let source = "unused=value\n";
    let parsed = Parser::new(source).parse();
    let (_, diagnostic_count, status) = inspect_parse_result(&parsed);
    assert_eq!(status, ParseStatus::Clean);
    assert_eq!(diagnostic_count, 0);

    let mut settings = LinterSettings::for_rules([Rule::UnusedAssignment]);
    settings
        .severity_overrides
        .insert(Rule::UnusedAssignment, Severity::Hint);
    settings.report_environment_style_names = true;

    let source_path_resolver = |_: &Path, _: &str| Vec::<PathBuf>::new();
    accepts_source_path_resolver(&source_path_resolver);
    let plugin_resolver = EmptyPluginResolver;
    let analysis = AnalysisRequest::from_parse_result(&parsed, source, &settings)
        .with_source_path_resolver(&source_path_resolver)
        .with_plugin_resolver(&plugin_resolver)
        .analyze();
    let _semantic = &analysis.semantic;
    assert_eq!(analysis.diagnostics.len(), 1);
    let (_, rule, severity) = inspect_diagnostic(&analysis.diagnostics[0]);
    assert_eq!(rule, Rule::UnusedAssignment);
    assert_eq!(severity, Severity::Hint);

    assert!(is_selected_rule(Rule::UnusedAssignment));
    let _resolver_companion_types = (
        PluginFramework::Other("custom".to_owned()),
        PluginRequestKind::Plugin,
        FileContract::default(),
    );
}

#[test]
fn fix_applicability_and_relint() {
    let source = "# !/bin/sh\n:\n";
    let settings = LinterSettings::for_rule(Rule::SpaceAfterHashBang);
    let parsed = Parser::new(source).parse();
    assert_eq!(parsed.status, ParseStatus::Clean);
    let diagnostics = AnalysisRequest::from_parse_result(&parsed, source, &settings).lint();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].fix.as_ref().unwrap().applicability(), Applicability::Unsafe);

    let safe = apply_fixes(source, &diagnostics, Applicability::Safe);
    assert_eq!(safe.fixes_applied, 0);
    assert_eq!(safe.code, source);
    let fixed = apply_fixes(source, &diagnostics, Applicability::Unsafe);
    assert_eq!(fixed.fixes_applied, 1);
    assert_eq!(fixed.code, "#!/bin/sh\n:\n");
    let reparsed = Parser::new(&fixed.code).parse();
    assert_eq!(reparsed.status, ParseStatus::Clean);
    assert!(AnalysisRequest::from_parse_result(&reparsed, &fixed.code, &settings).lint().is_empty());
}

#[test]
fn selection_layers_preserve_specific_rules_and_reject_unknown_codes() {
    let parse = |values: &[&str]| {
        parse_rule_selectors(&values.iter().map(|value| (*value).to_owned()).collect::<Vec<_>>(), "lint").unwrap()
    };
    let selected = apply_rule_selector_layer(
        RuleSet::all(),
        Some(&parse(&["C006"])),
        &parse(&["S074"]),
        &parse(&["C"]),
    );
    assert_eq!(selected.iter().collect::<Vec<_>>(), vec![Rule::UndefinedVariable, Rule::AmpersandSemicolon]);
    let extended = apply_rule_selector_layer(
        selected, None, &parse(&["C001"]), &parse(&["S074"]),
    );
    assert!(extended.contains(Rule::UndefinedVariable));
    assert!(extended.contains(Rule::UnusedAssignment));
    assert!(!extended.contains(Rule::AmpersandSemicolon));
    assert!("C999999".parse::<RuleSelector>().is_err());
}
