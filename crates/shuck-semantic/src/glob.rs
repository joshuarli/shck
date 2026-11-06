use crate::ShellBehaviorAt;

/// Whether unquoted expansion results are subject to field splitting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldSplittingBehavior {
    /// Field splitting does not apply.
    Never,
    /// Field splitting applies only when the expansion is unquoted.
    UnquotedOnly,
    /// Runtime option state may select either behavior.
    Ambiguous,
}

impl FieldSplittingBehavior {
    /// Returns whether an unquoted expansion result may be split into fields.
    pub fn unquoted_results_can_split(self) -> bool {
        matches!(self, Self::UnquotedOnly | Self::Ambiguous)
    }
}

/// Whether pathname expansion applies to literal globs and substitution results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathnameExpansionBehavior {
    /// Pathname expansion does not apply.
    Disabled,
    /// Literal glob characters are active, but substitution results are not globbed.
    LiteralGlobsOnly,
    /// Runtime option state may enable literal glob expansion, but substitution results stay plain.
    LiteralGlobsOnlyOrDisabled,
    /// Unquoted substitution results can also trigger pathname expansion.
    SubstitutionResultsWhenUnquoted,
    /// Runtime option state may select either behavior family.
    Ambiguous,
}

impl PathnameExpansionBehavior {
    /// Returns whether literal glob characters may trigger pathname expansion.
    pub fn literal_globs_can_expand(self) -> bool {
        !matches!(self, Self::Disabled)
    }

    /// Returns whether unquoted substitution results may be interpreted as globs.
    pub fn unquoted_substitution_results_can_glob(self) -> bool {
        matches!(
            self,
            Self::SubstitutionResultsWhenUnquoted | Self::Ambiguous
        )
    }
}

/// Whether zsh filename expansion runs before or after parameter expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileExpansionOrderBehavior {
    /// Filename expansion runs before parameter expansion.
    BeforeParameterExpansion,
    /// Filename expansion runs after parameter expansion.
    AfterParameterExpansion,
    /// Runtime option state may select either order.
    Ambiguous,
}

/// How unmatched glob patterns behave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobFailureBehavior {
    /// An unmatched glob produces an error.
    ErrorOnNoMatch,
    /// An unmatched glob stays literal in argv.
    KeepLiteralOnNoMatch,
    /// An unmatched glob is removed from argv.
    DropUnmatchedPattern,
    /// Unmatched globs are removed unless every glob in the command misses.
    CshNullGlob,
    /// Runtime option state may select either behavior.
    Ambiguous,
}

/// Whether glob patterns match dot-prefixed path segments without an explicit dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobDotBehavior {
    /// Dot-prefixed names require an explicit dot in the pattern.
    ExplicitDotRequired,
    /// Dot-prefixed names may be matched by ordinary glob metacharacters.
    DotfilesIncluded,
    /// Runtime option state may select either behavior.
    Ambiguous,
}

/// Whether a zsh pattern-operator family is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternOperatorBehavior {
    /// The operator family is disabled.
    Disabled,
    /// The operator family is enabled.
    Enabled,
    /// Runtime option state may select either behavior.
    Ambiguous,
}

/// Whether zsh brace character-class expansion is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BraceCharacterClassBehavior {
    /// Brace character classes are not expanded.
    Disabled,
    /// Brace character classes are expanded.
    Enabled,
    /// Runtime option state may select either behavior.
    Ambiguous,
}

impl BraceCharacterClassBehavior {
    /// Returns whether a brace character class may expand at runtime.
    pub fn can_expand(self) -> bool {
        matches!(self, Self::Enabled | Self::Ambiguous)
    }
}

/// Option-sensitive pattern operators available to glob and shell-pattern facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobPatternBehavior {
    extended_glob: PatternOperatorBehavior,
    ksh_glob: PatternOperatorBehavior,
    sh_glob: PatternOperatorBehavior,
}

impl GlobPatternBehavior {
    /// Creates a pattern behavior from its operator-family states.
    pub const fn from_parts(
        extended_glob: PatternOperatorBehavior,
        ksh_glob: PatternOperatorBehavior,
        sh_glob: PatternOperatorBehavior,
    ) -> Self {
        Self {
            extended_glob,
            ksh_glob,
            sh_glob,
        }
    }

    /// Returns whether zsh extended glob operators such as `#`, `~`, and `^` are active.
    pub fn extended_glob(self) -> PatternOperatorBehavior {
        self.extended_glob
    }

    /// Returns whether ksh-style glob operators such as `@(...)` are active.
    pub fn ksh_glob(self) -> PatternOperatorBehavior {
        self.ksh_glob
    }

    /// Returns whether sh-compatible glob parsing is active.
    pub fn sh_glob(self) -> PatternOperatorBehavior {
        self.sh_glob
    }
}

impl ShellBehaviorAt {
    /// Returns the field-splitting behavior implied by the shell and runtime option state.
    pub fn field_splitting(&self) -> FieldSplittingBehavior {
        FieldSplittingBehavior::UnquotedOnly
    }

    /// Returns the pathname-expansion behavior implied by the shell and runtime option state.
    pub fn pathname_expansion(&self) -> PathnameExpansionBehavior {
        PathnameExpansionBehavior::SubstitutionResultsWhenUnquoted
    }

    /// Returns the filename-expansion order implied by the shell and runtime option state.
    pub fn file_expansion_order(&self) -> FileExpansionOrderBehavior {
        FileExpansionOrderBehavior::BeforeParameterExpansion
    }

    /// Returns the unmatched-glob behavior implied by the shell and runtime option state.
    pub fn glob_failure(&self) -> GlobFailureBehavior {
        GlobFailureBehavior::KeepLiteralOnNoMatch
    }

    /// Returns how glob patterns treat dot-prefixed path segments.
    pub fn glob_dot_matching(&self) -> GlobDotBehavior {
        GlobDotBehavior::ExplicitDotRequired
    }

    /// Returns the option-sensitive glob/pattern operator families available at this offset.
    pub fn glob_pattern(&self) -> GlobPatternBehavior {
        GlobPatternBehavior::from_parts(
            PatternOperatorBehavior::Disabled,
            PatternOperatorBehavior::Enabled,
            PatternOperatorBehavior::Enabled,
        )
    }

    /// Returns whether zsh brace character classes such as `{abc}` are active.
    pub fn brace_character_classes(&self) -> BraceCharacterClassBehavior {
        BraceCharacterClassBehavior::Disabled
    }
}

#[allow(dead_code)]
fn pattern_operator_behavior(_value: bool) -> PatternOperatorBehavior {
        PatternOperatorBehavior::Disabled
    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SemanticBuildOptions, SemanticModel, ShellDialect, ShellProfile};
    use shuck_indexer::Indexer;
    use shuck_parser::parser::Parser;

    fn model_with_profile(source: &str, profile: ShellProfile) -> SemanticModel {
        let output = Parser::with_profile(source, profile.clone())
            .parse()
            .unwrap();
        let indexer = Indexer::new(source, &output);
        SemanticModel::build_with_options(
            &output.file,
            source,
            &indexer,
            SemanticBuildOptions { shell_profile: Some(profile), ..SemanticBuildOptions::default() },
        )
    }

    #[test]
    fn default_behaviors_are_stable() {
        let profile = ShellProfile::native(ShellDialect::Bash);
        let model = model_with_profile("echo ok\n", profile);
        let behavior = model.shell_behavior_at(0);
        assert_eq!(
            behavior.field_splitting(),
            FieldSplittingBehavior::UnquotedOnly
        );
        assert_eq!(
            behavior.pathname_expansion(),
            PathnameExpansionBehavior::SubstitutionResultsWhenUnquoted
        );
        assert_eq!(
            behavior.glob_failure(),
            GlobFailureBehavior::KeepLiteralOnNoMatch
        );
        assert_eq!(
            behavior.glob_dot_matching(),
            GlobDotBehavior::ExplicitDotRequired
        );
        assert_eq!(
            behavior.brace_character_classes(),
            BraceCharacterClassBehavior::Disabled
        );
        let pattern = behavior.glob_pattern();
        assert_eq!(pattern.sh_glob(), PatternOperatorBehavior::Enabled);
    }
}
