pub(crate) mod common;
pub mod correctness;
pub mod performance;
pub mod portability;
pub mod security;
pub mod style;

#[cfg(test)]
mod parity_tests;

#[cfg(test)]
mod architecture_tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    const RULE_DIRS: &[&str] = &[
        "correctness",
        "performance",
        "portability",
        "security",
        "style",
    ];
    const FORBIDDEN_TOKENS: &[&str] = &[
        "WordPart",
        "WordPartNode",
        "ConditionalExpr",
        "PatternPart",
        "ParameterExpansionSyntax",
        "ConditionalCommand",
        "BourneParameterExpansion",
        "iter_commands",
        "query::",
    ];

    /// Rule-local helper files that predate the facts layer and still reach into AST
    /// traversal types. Each entry is debt: the goal is to move the structural work into
    /// `src/facts/` and delete the entry. Paths are relative to `src/rules/`.
    const TRAVERSAL_TOKEN_ALLOWLIST: &[&str] = &[
        // S001's safe-value classifier walks word parts, parameter expansions, and
        // conditional operands to decide whether an unquoted expansion can split or glob.
        // It should become a word-safety fact under `src/facts/` that S001 (and any other
        // quoting rule) consumes, at which point this entry goes away.
        "style/unquoted_expansion/safe_value.rs",
    ];

    fn collect_rule_sources(dir: &Path, out: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(dir)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()));
        for entry in entries {
            let path = entry
                .unwrap_or_else(|error| {
                    panic!("failed to read entry in {}: {error}", dir.display())
                })
                .path();
            if path.is_dir() {
                collect_rule_sources(&path, out);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }

    #[test]
    fn rule_modules_avoid_direct_ast_traversal_tokens() {
        let rules_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules");
        let mut violations = Vec::new();
        let mut unused_allowlist = TRAVERSAL_TOKEN_ALLOWLIST.to_vec();

        for dir in RULE_DIRS {
            let category_dir = rules_root.join(dir);
            let mut paths = Vec::new();
            collect_rule_sources(&category_dir, &mut paths);
            paths.sort();

            for path in paths {
                // The category `mod.rs` only declares modules and fixture test cases.
                if path == category_dir.join("mod.rs") {
                    continue;
                }
                let relative = path
                    .strip_prefix(&rules_root)
                    .expect("rule source lives under src/rules")
                    .to_string_lossy()
                    .replace('\\', "/");

                let source = fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
                let found = FORBIDDEN_TOKENS
                    .iter()
                    .copied()
                    .filter(|token| source.contains(token))
                    .collect::<Vec<_>>();

                if TRAVERSAL_TOKEN_ALLOWLIST.contains(&relative.as_str()) {
                    // An allowlisted file that has been cleaned up should drop its entry.
                    if !found.is_empty() {
                        unused_allowlist.retain(|entry| *entry != relative);
                    }
                    continue;
                }
                for token in found {
                    violations.push(format!("{} contains `{token}`", path.display()));
                }
            }
        }

        assert!(
            violations.is_empty(),
            "rule files should rely on fact/shared helper APIs instead of direct AST traversal:\n{}",
            violations.join("\n"),
        );
        assert!(
            unused_allowlist.is_empty(),
            "stale TRAVERSAL_TOKEN_ALLOWLIST entries (file moved or no longer needs an exemption):\n{}",
            unused_allowlist.join("\n"),
        );
    }

    #[test]
    fn rules_common_has_no_query_module() {
        let query_module = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules/common/query.rs");

        assert!(
            !query_module.exists(),
            "rule-facing traversal helpers must live in facts, not rules/common/query.rs",
        );
    }

    #[test]
    fn facts_traversal_helpers_stay_private() {
        let traversal_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/facts/traversal.rs");
        let source = fs::read_to_string(&traversal_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", traversal_path.display()));
        let forbidden_visibility = [
            "pub(crate) struct CommandVisit",
            "pub(crate) fn visit_arithmetic_words",
            "pub(crate) fn visit_var_ref_subscript_words",
            "pub(crate) fn visit_subscript_words",
        ];
        let violations = forbidden_visibility
            .iter()
            .copied()
            .filter(|token| source.contains(token))
            .collect::<Vec<_>>();

        assert!(
            violations.is_empty(),
            "facts traversal helpers should stay private to the facts module:\n{}",
            violations.join("\n"),
        );
    }

}
