//! Resolution of a loaded [`ShuckConfig`] plus ordered override layers into
//! the compiled settings consumed by the linter and formatter.
//!
//! The CLI and the language server both resolve settings through
//! [`SettingsResolver`] so a given `.shuck.toml` yields the same rule set,
//! rule options, contracts, per-file ignores, and per-file shells in every
//! front end. The WebAssembly bindings cannot depend on this crate, so they
//! reuse the same lower-level building blocks instead
//! ([`shuck_linter::apply_rule_selector_layer`],
//! [`ShellFormatOptions::with_patch`], and
//! [`FormatDialect::for_shell_name`]).

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Result, anyhow};
use shuck_formatter::{ShellDialect as FormatDialect, ShellFormatOptions};
pub use shuck_linter::parse_rule_selectors;
use shuck_linter::{
    AmbientContractActivation, AmbientContractConfig, AmbientContractEffects, AmbientContractSpec,
    AmbientFunctionContractSpec, CompiledPerFileIgnoreList, CompiledPerFileShellList,
    LinterRuleOptions, LinterSettings, PerFileIgnore, PerFileShell, ResolvedAmbientContracts,
    RuleSelector, RuleSet, S083FunctionDocRequirement, ShellDialect, apply_rule_selector_layer,
};

use crate::{
    FormatExclusions, LintContractActivationTypeConfig, LintContractWhenConfig,
    LintContractsConfig, LintCustomContractConfig, LintRuleOptionsConfig, S083RequireForConfig,
    ShuckConfig, apply_config_overrides,
};

mod source;

pub use source::{NativeSourceResolver, resolve_source_ref_paths, source_ref_candidate_paths};

/// Error reported when `[format].indent-width` is zero.
pub const CONFIG_INDENT_WIDTH_ERROR: &str = "`[format].indent-width` must be at least 1";

/// How resolution reacts to invalid settings values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvePolicy {
    /// Fail on the first invalid value. Used by the CLI.
    Strict,
    /// Skip invalid values and record a warning for each one. Used by the
    /// language server, which should keep working while a config is edited.
    Lenient,
}

/// One ordered layer of settings applied on top of the project config.
#[derive(Debug, Clone)]
pub enum SettingsOverride {
    /// Config-shaped overrides, such as language-server client options.
    ///
    /// Rule selection, per-file ignores, per-file shells, and zsh plugin
    /// settings are layered with the same replace/extend semantics as the
    /// project config. Every other field replaces the value below it.
    Config(Box<ShuckConfig>),
    /// Rule-selection and zsh-plugin overrides that were already parsed,
    /// such as command-line flags.
    Parsed(Box<ParsedOverride>),
}

/// Pre-parsed rule-selection overrides.
#[derive(Debug, Clone, Default)]
pub struct ParsedOverride {
    /// Rule selection, fixability, and per-file overrides.
    pub rules: RuleSelectionLayer,
}

/// One layer of rule-selection settings.
///
/// `Some` replace-style fields replace everything below the layer, and the
/// matching `extend_*` fields are appended afterwards.
#[derive(Debug, Clone, Default)]
pub struct RuleSelectionLayer {
    /// Selectors that replace the enabled rule set.
    pub select: Option<Vec<RuleSelector>>,
    /// Selectors removed from the enabled rule set.
    pub ignore: Vec<RuleSelector>,
    /// Selectors added to the enabled rule set.
    pub extend_select: Vec<RuleSelector>,
    /// Per-file ignores that replace earlier layers.
    pub per_file_ignores: Option<Vec<PerFileIgnoreSpec>>,
    /// Per-file ignores appended to earlier layers.
    pub extend_per_file_ignores: Vec<PerFileIgnoreSpec>,
    /// Per-file shells that replace earlier layers.
    pub per_file_shell: Option<Vec<PerFileShell>>,
    /// Per-file shells appended to earlier layers.
    pub extend_per_file_shell: Vec<PerFileShell>,
    /// Selectors that replace the fixable rule set.
    pub fixable: Option<Vec<RuleSelector>>,
    /// Selectors removed from the fixable rule set.
    pub unfixable: Vec<RuleSelector>,
    /// Selectors added to the fixable rule set.
    pub extend_fixable: Vec<RuleSelector>,
}

/// A glob pattern paired with the selectors ignored for matching files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerFileIgnoreSpec {
    /// Glob pattern matched against file paths.
    pub pattern: String,
    /// Selectors ignored for matching files.
    pub selectors: Vec<RuleSelector>,
}

impl PerFileIgnoreSpec {
    fn into_ignore(self) -> PerFileIgnore {
        let rules = self
            .selectors
            .into_iter()
            .fold(RuleSet::EMPTY, |rules, selector| {
                rules.union(&selector.into_rule_set())
            });
        PerFileIgnore::new(self.pattern, rules)
    }
}

/// Lint settings produced by [`SettingsResolver::resolve_lint`].
#[derive(Debug, Clone)]
pub struct ResolvedLintSettings {
    /// Linter settings with enabled rules, rule options, ambient contracts,
    /// and per-file ignores applied. The shell is left as
    /// [`ShellDialect::Unknown`]; see [`Self::linter_for_path`].
    pub linter: LinterSettings,
    /// Rules whose fixes may be applied.
    pub fixable_rules: RuleSet,
    /// Per-file ignores before glob compilation, in layer order.
    pub per_file_ignores: Vec<PerFileIgnore>,
    /// Per-file shells before glob compilation, in layer order.
    pub per_file_shell: Vec<PerFileShell>,
    /// Compiled per-file shell matcher.
    pub compiled_per_file_shell: Arc<CompiledPerFileShellList>,
    /// Extra directories searched for `# shuck: source=` targets.
    pub source_paths: Vec<String>,
    /// Whether `lint=true` source directives lint the resolved target.
    pub lint_sources: bool,
}

impl Default for ResolvedLintSettings {
    fn default() -> Self {
        let linter = LinterSettings::default();
        Self {
            linter,
            fixable_rules: RuleSet::all(),
            per_file_ignores: Vec::new(),
            per_file_shell: Vec::new(),
            compiled_per_file_shell: Arc::default(),
            source_paths: Vec::new(),
            lint_sources: true,
        }
    }
}

impl ResolvedLintSettings {
    /// Returns a resolver for `# shuck: source=` targets that searches the
    /// configured `source-paths`, with relative roots anchored at `root_base`
    /// (normally the project root).
    pub fn source_resolver(&self, root_base: PathBuf) -> NativeSourceResolver {
        NativeSourceResolver::new(root_base, self.source_paths.clone())
    }

    /// Returns linter settings with the per-file shell for `path` applied.
    pub fn linter_for_path(&self, path: Option<&Path>) -> LinterSettings {
        let mut linter = self.linter.clone();
        linter.shell = path
            .and_then(|path| self.compiled_per_file_shell.shell_for_path(path))
            .unwrap_or(ShellDialect::Unknown);
        linter
    }
}

/// Formatter settings produced by [`SettingsResolver::resolve_format`].
#[derive(Debug, Clone, Default)]
pub struct ResolvedFormatSettings {
    /// Formatter options from the `[format]` table.
    pub options: ShellFormatOptions,
    /// Compiled `[format].exclude` patterns.
    pub exclusions: FormatExclusions,
    /// Per-file shells before glob compilation, in layer order.
    pub per_file_shell: Vec<PerFileShell>,
    /// Compiled per-file shell matcher.
    pub compiled_per_file_shell: Arc<CompiledPerFileShellList>,
}

impl ResolvedFormatSettings {
    /// Returns formatter options for `path`, applying its per-file shell when
    /// no explicit dialect was requested.
    pub fn options_for_path(&self, path: &Path) -> Result<ShellFormatOptions> {
        format_options_for_path(&self.options, &self.compiled_per_file_shell, path)
    }
}

/// Applies the per-file shell for `path` to `options` when their dialect is
/// [`FormatDialect::Auto`].
pub fn format_options_for_path(
    options: &ShellFormatOptions,
    per_file_shell: &CompiledPerFileShellList,
    path: &Path,
) -> Result<ShellFormatOptions> {
    if options.dialect() != FormatDialect::Auto {
        return Ok(options.clone());
    }

    let Some(shell) = per_file_shell.shell_for_path_checked(path)? else {
        return Ok(options.clone());
    };
    Ok(options
        .clone()
        .with_dialect(format_dialect_for_shell(shell)?))
}

/// Maps a linter shell dialect onto the formatter dialect that handles it.
pub fn format_dialect_for_shell(shell: ShellDialect) -> Result<FormatDialect> {
    match shell {
        ShellDialect::Unknown => Err(anyhow!("unknown per-file shell dialect")),
        shell => FormatDialect::for_shell_name(shell.as_str()).ok_or_else(|| {
            anyhow!(
                "per-file shell `{}` is not supported by the formatter; use `mksh` for mksh files",
                shell.as_str()
            )
        }),
    }
}

/// Resolves a project config plus override layers into compiled settings.
#[derive(Debug)]
pub struct SettingsResolver<'a> {
    project_root: &'a Path,
    config: &'a ShuckConfig,
    overrides: &'a [SettingsOverride],
    policy: ResolvePolicy,
    per_file_shell: Option<Vec<PerFileShell>>,
    warnings: Vec<String>,
}

impl<'a> SettingsResolver<'a> {
    /// Creates a resolver for `config`. Path globs are compiled relative to
    /// `project_root`.
    pub fn new(project_root: &'a Path, config: &'a ShuckConfig, policy: ResolvePolicy) -> Self {
        Self {
            project_root,
            config,
            overrides: &[],
            policy,
            per_file_shell: None,
            warnings: Vec::new(),
        }
    }

    /// Adds override layers, applied in order on top of the project config.
    #[must_use]
    pub fn with_overrides(mut self, overrides: &'a [SettingsOverride]) -> Self {
        self.overrides = overrides;
        self.per_file_shell = None;
        self
    }

    /// Warnings collected for values skipped under [`ResolvePolicy::Lenient`].
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Resolves rule selection, rule options, contracts, and per-file settings.
    pub fn resolve_lint(&mut self) -> Result<ResolvedLintSettings> {
        let mut rule_layers = vec![self.parse_config_rule_layer(self.config)?];
        for layer in self.overrides {
            match layer {
                SettingsOverride::Config(config) => {
                    rule_layers.push(self.parse_config_rule_layer(config)?);
                }
                SettingsOverride::Parsed(parsed) => {
                    rule_layers.push(parsed.rules.clone());
                }
            }
        }
        let merged = self.merged_config();
        let rule_options = self.linter_rule_options(merged.lint.rule_options.as_ref())?;

        let mut enabled_rules = LinterSettings::default_rules();
        let mut fixable_rules = RuleSet::all();
        let mut per_file_ignores = Vec::new();
        let mut per_file_shell = Vec::new();
        for layer in rule_layers {
            enabled_rules = apply_rule_selector_layer(
                enabled_rules,
                layer.select.as_deref(),
                &layer.extend_select,
                &layer.ignore,
            );
            fixable_rules = apply_rule_selector_layer(
                fixable_rules,
                layer.fixable.as_deref(),
                &layer.extend_fixable,
                &layer.unfixable,
            );
            per_file_ignores = apply_extend_layer(
                per_file_ignores,
                layer.per_file_ignores.map(|specs| {
                    specs
                        .into_iter()
                        .map(PerFileIgnoreSpec::into_ignore)
                        .collect()
                }),
                layer
                    .extend_per_file_ignores
                    .into_iter()
                    .map(PerFileIgnoreSpec::into_ignore)
                    .collect(),
            );
            per_file_shell = apply_extend_layer(
                per_file_shell,
                layer.per_file_shell,
                layer.extend_per_file_shell,
            );
        }
        self.per_file_shell = Some(per_file_shell.clone());

        let compiled_per_file_ignores = self
            .recover(CompiledPerFileIgnoreList::resolve(
                self.project_root.to_path_buf(),
                per_file_ignores.clone(),
            ))?
            .unwrap_or_default();
        let compiled_per_file_shell = self.compile_per_file_shell(per_file_shell.clone())?;
        let ambient_contracts = Arc::new(
            self.recover(resolve_ambient_contracts(
                self.project_root,
                merged.lint.contracts.as_ref(),
            ))?
            .unwrap_or_default(),
        );
        let mut linter = LinterSettings::for_rules(enabled_rules.iter());
        linter.ambient_contracts = ambient_contracts;
        linter.per_file_ignores = Arc::new(compiled_per_file_ignores);
        linter.rule_options = rule_options;

        Ok(ResolvedLintSettings {
            linter,
            fixable_rules,
            per_file_ignores,
            per_file_shell,
            compiled_per_file_shell: Arc::new(compiled_per_file_shell),
            source_paths: merged.lint.source_paths.clone().unwrap_or_default(),
            lint_sources: merged.lint.lint_sources.unwrap_or(true),
        })
    }

    /// Resolves `[format]` options, format exclusions, and per-file shells.
    ///
    /// Only the settings the formatter reads are validated, so invalid lint
    /// selectors do not block formatting.
    pub fn resolve_format(&mut self) -> Result<ResolvedFormatSettings> {
        let merged = self.merged_config();
        let mut patch = self.recover(merged.format.to_patch())?.unwrap_or_default();
        let exclusions = self
            .recover(merged.format.compile_exclusions(self.project_root))?
            .unwrap_or_default();
        let per_file_shell = self.resolved_per_file_shell()?;
        let compiled_per_file_shell = self.compile_per_file_shell(per_file_shell.clone())?;
        if patch.indent_width == Some(0) {
            self.recover::<()>(Err(anyhow!(CONFIG_INDENT_WIDTH_ERROR)))?;
            patch.indent_width = None;
        }

        Ok(ResolvedFormatSettings {
            options: ShellFormatOptions::default().with_patch(&patch),
            exclusions,
            per_file_shell,
            compiled_per_file_shell: Arc::new(compiled_per_file_shell),
        })
    }

    /// Returns `Ok(Some(value))` on success. On failure, returns the error
    /// under [`ResolvePolicy::Strict`] and records a warning and returns
    /// `Ok(None)` under [`ResolvePolicy::Lenient`].
    fn recover<T>(&mut self, result: Result<T>) -> Result<Option<T>> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(error) => match self.policy {
                ResolvePolicy::Strict => Err(error),
                ResolvePolicy::Lenient => {
                    self.warnings.push(format!("{error:#}"));
                    Ok(None)
                }
            },
        }
    }

    fn merged_config(&self) -> Cow<'a, ShuckConfig> {
        let mut merged = Cow::Borrowed(self.config);
        for layer in self.overrides {
            if let SettingsOverride::Config(overrides) = layer {
                apply_config_overrides(merged.to_mut(), ShuckConfig::clone(overrides));
            }
        }
        merged
    }

    fn compile_per_file_shell(
        &mut self,
        per_file_shell: Vec<PerFileShell>,
    ) -> Result<CompiledPerFileShellList> {
        Ok(self
            .recover(CompiledPerFileShellList::resolve(
                self.project_root.to_path_buf(),
                per_file_shell,
            ))?
            .unwrap_or_default())
    }

    fn resolved_per_file_shell(&mut self) -> Result<Vec<PerFileShell>> {
        if let Some(per_file_shell) = &self.per_file_shell {
            return Ok(per_file_shell.clone());
        }

        let mut per_file_shell = Vec::new();
        let (replace, extend) = self.parse_config_per_file_shell(self.config)?;
        per_file_shell = apply_extend_layer(per_file_shell, replace, extend);
        for layer in self.overrides {
            let (replace, extend) = match layer {
                SettingsOverride::Config(config) => self.parse_config_per_file_shell(config)?,
                SettingsOverride::Parsed(parsed) => (
                    parsed.rules.per_file_shell.clone(),
                    parsed.rules.extend_per_file_shell.clone(),
                ),
            };
            per_file_shell = apply_extend_layer(per_file_shell, replace, extend);
        }
        self.per_file_shell = Some(per_file_shell.clone());
        Ok(per_file_shell)
    }

    fn parse_config_rule_layer(&mut self, config: &ShuckConfig) -> Result<RuleSelectionLayer> {
        let lint = &config.lint;
        let per_file_shell = match config.per_file_shell.as_ref() {
            Some(patterns) => Some(self.parse_per_file_shell_map(patterns, "per-file-shell")?),
            None => lint
                .per_file_shell
                .as_ref()
                .map(|patterns| self.parse_per_file_shell_map(patterns, "lint.per-file-shell"))
                .transpose()?,
        };

        Ok(RuleSelectionLayer {
            select: self.parse_selector_list(lint.select.as_deref(), "lint.select")?,
            ignore: self
                .parse_selector_list(lint.ignore.as_deref(), "lint.ignore")?
                .unwrap_or_default(),
            extend_select: self
                .parse_selector_list(lint.extend_select.as_deref(), "lint.extend-select")?
                .unwrap_or_default(),
            per_file_ignores: lint
                .per_file_ignores
                .as_ref()
                .map(|patterns| self.parse_per_file_ignore_map(patterns, "lint.per-file-ignores"))
                .transpose()?,
            extend_per_file_ignores: lint
                .extend_per_file_ignores
                .as_ref()
                .map(|patterns| {
                    self.parse_per_file_ignore_map(patterns, "lint.extend-per-file-ignores")
                })
                .transpose()?
                .unwrap_or_default(),
            per_file_shell,
            extend_per_file_shell: lint
                .extend_per_file_shell
                .as_ref()
                .map(|patterns| {
                    self.parse_per_file_shell_map(patterns, "lint.extend-per-file-shell")
                })
                .transpose()?
                .unwrap_or_default(),
            fixable: self.parse_selector_list(lint.fixable.as_deref(), "lint.fixable")?,
            unfixable: self
                .parse_selector_list(lint.unfixable.as_deref(), "lint.unfixable")?
                .unwrap_or_default(),
            extend_fixable: self
                .parse_selector_list(lint.extend_fixable.as_deref(), "lint.extend-fixable")?
                .unwrap_or_default(),
        })
    }

    fn parse_config_per_file_shell(
        &mut self,
        config: &ShuckConfig,
    ) -> Result<(Option<Vec<PerFileShell>>, Vec<PerFileShell>)> {
        let replace = match config.per_file_shell.as_ref() {
            Some(patterns) => Some(self.parse_per_file_shell_map(patterns, "per-file-shell")?),
            None => config
                .lint
                .per_file_shell
                .as_ref()
                .map(|patterns| self.parse_per_file_shell_map(patterns, "lint.per-file-shell"))
                .transpose()?,
        };
        let extend = config
            .lint
            .extend_per_file_shell
            .as_ref()
            .map(|patterns| self.parse_per_file_shell_map(patterns, "lint.extend-per-file-shell"))
            .transpose()?
            .unwrap_or_default();
        Ok((replace, extend))
    }

    /// Parses a selector list. Under [`ResolvePolicy::Lenient`] a list with
    /// any invalid selector is skipped as a whole.
    fn parse_selector_list(
        &mut self,
        selectors: Option<&[String]>,
        scope: &str,
    ) -> Result<Option<Vec<RuleSelector>>> {
        let Some(selectors) = selectors else {
            return Ok(None);
        };
        self.recover(parse_rule_selectors(selectors, scope))
    }

    /// Parses per-file ignores. Under [`ResolvePolicy::Lenient`] entries with
    /// invalid selectors are skipped.
    fn parse_per_file_ignore_map(
        &mut self,
        patterns: &BTreeMap<String, Vec<String>>,
        scope: &str,
    ) -> Result<Vec<PerFileIgnoreSpec>> {
        let mut specs = Vec::with_capacity(patterns.len());
        for (pattern, selectors) in patterns {
            if let Some(selectors) = self.recover(parse_rule_selectors(selectors, scope))? {
                specs.push(PerFileIgnoreSpec {
                    pattern: pattern.clone(),
                    selectors,
                });
            }
        }
        Ok(specs)
    }

    /// Parses per-file shells. Under [`ResolvePolicy::Lenient`] entries with
    /// unknown shells are skipped.
    fn parse_per_file_shell_map(
        &mut self,
        patterns: &BTreeMap<String, String>,
        scope: &str,
    ) -> Result<Vec<PerFileShell>> {
        let mut entries = Vec::with_capacity(patterns.len());
        for (pattern, shell) in patterns {
            let parsed = parse_shell_dialect(shell)
                .map_err(|err| anyhow!("invalid {scope} shell `{shell}`: {err}"));
            if let Some(shell) = self.recover(parsed)? {
                entries.push(PerFileShell::new(pattern.clone(), shell));
            }
        }
        Ok(entries)
    }

    fn linter_rule_options(
        &mut self,
        config: Option<&LintRuleOptionsConfig>,
    ) -> Result<LinterRuleOptions> {
        let mut options = LinterRuleOptions::default();
        let Some(config) = config else {
            return Ok(options);
        };

        if let Some(c001) = &config.c001
            && let Some(value) = c001.treat_indirect_expansion_targets_as_used
        {
            options.c001.treat_indirect_expansion_targets_as_used = value;
        }
        if let Some(c063) = &config.c063
            && let Some(value) = c063.report_unreached_nested_definitions
        {
            options.c063.report_unreached_nested_definitions = value;
        }
        if let Some(s078) = &config.s078
            && let Some(value) = &s078.allowed_shells
        {
            options.s078.allowed_shells.clone_from(value);
        }
        if let Some(s079) = &config.s079 {
            if let Some(value) = &s079.allowed_forms {
                options.s079.allowed_forms.clone_from(value);
            }
            if let Some(value) = &s079.allowed_paths {
                options.s079.allowed_paths.clone_from(value);
            }
        }
        if let Some(s080) = &config.s080 {
            if let Some(value) = s080.max_lines {
                options.s080.max_lines = value;
            }
            if let Some(value) = &s080.count
                && self.recover(validate_s080_count_mode(value))?.is_some()
            {
                options.s080.count.clone_from(value);
            }
        }
        if let Some(s081) = &config.s081
            && let Some(value) = s081.ignore_shebang_only_files
        {
            options.s081.ignore_shebang_only_files = value;
        }
        if let Some(s082) = &config.s082 {
            if let Some(value) = &s082.kinds {
                options.s082.kinds.clone_from(value);
            }
            if let Some(value) = s082.require_owner {
                options.s082.require_owner = value;
            }
            if let Some(value) = s082.require_message {
                options.s082.require_message = value;
            }
        }
        if let Some(s083) = &config.s083 {
            if let Some(value) = s083.require_for {
                options.s083.require_for = match value {
                    S083RequireForConfig::All => S083FunctionDocRequirement::All,
                    S083RequireForConfig::Exported => S083FunctionDocRequirement::Exported,
                    S083RequireForConfig::Long => S083FunctionDocRequirement::Long,
                    S083RequireForConfig::Parameterized => {
                        S083FunctionDocRequirement::Parameterized
                    }
                };
            }
            if let Some(value) = s083.long_function_line_threshold {
                options.s083.long_function_line_threshold = value;
            }
        }
        if let Some(s084) = &config.s084 {
            if let Some(value) = s084.require_globals {
                options.s084.require_globals = value;
            }
            if let Some(value) = s084.require_arguments {
                options.s084.require_arguments = value;
            }
            if let Some(value) = s084.require_outputs {
                options.s084.require_outputs = value;
            }
            if let Some(value) = s084.require_returns {
                options.s084.require_returns = value;
            }
        }
        if let Some(s085) = &config.s085 {
            if let Some(value) = s085.non_trivial_line_threshold {
                options.s085.non_trivial_line_threshold = value;
            }
            if let Some(value) = s085.non_trivial_function_count {
                options.s085.non_trivial_function_count = value;
            }
            if let Some(value) = &s085.main_name {
                options.s085.main_name.clone_from(value);
            }
        }
        if let Some(c158) = &config.c158 {
            if let Some(value) = c158.treat_readonly_as_documented {
                options.c158.treat_readonly_as_documented = value;
            }
            if let Some(value) = c158.treat_export_as_intentional {
                options.c158.treat_export_as_intentional = value;
            }
        }
        if let Some(c159) = &config.c159
            && let Some(value) = c159.allow_conditional_init
        {
            options.c159.allow_conditional_init = value;
        }
        if let Some(c160) = &config.c160
            && let Some(value) = &c160.allowed_anchors
        {
            options.c160.allowed_anchors.clone_from(value);
        }
        if let Some(c161) = &config.c161
            && let Some(value) = c161.ignore_after_source
        {
            options.c161.ignore_after_source = value;
        }

        Ok(options)
    }
}

fn parse_shell_dialect(value: &str) -> Result<ShellDialect> {
    let shell = ShellDialect::from_name(value);
    if shell == ShellDialect::Unknown {
        return Err(anyhow!(
            "expected shell dialect to be one of sh, bash, dash, ksh, mksh"
        ));
    }

    Ok(shell)
}

fn validate_s080_count_mode(value: &str) -> Result<()> {
    match value.trim().to_ascii_lowercase().as_str() {
        "physical" | "non-comment-non-blank" => Ok(()),
        _ => Err(anyhow!(
            "unsupported `[lint.rule-options.s080].count` value `{}`; expected one of: physical, non-comment-non-blank",
            value
        )),
    }
}

fn apply_extend_layer<T>(current: Vec<T>, replace: Option<Vec<T>>, extend: Vec<T>) -> Vec<T> {
    let mut merged = replace.unwrap_or(current);
    merged.extend(extend);
    merged
}

/// Compiles `[lint.contracts]` into resolved ambient contracts.
pub fn resolve_ambient_contracts(
    project_root: &Path,
    config: Option<&LintContractsConfig>,
) -> Result<ResolvedAmbientContracts> {
    let Some(config) = config else {
        return Ok(ResolvedAmbientContracts::default());
    };

    let mut ambient = AmbientContractConfig {
        well_known: config.well_known.unwrap_or(true),
        disabled: config.disabled.clone().unwrap_or_default(),
        custom: Vec::new(),
    };

    for contract in config.custom.as_deref().unwrap_or(&[]) {
        ambient
            .custom
            .push(ambient_contract_spec_from_config(contract)?);
    }

    ResolvedAmbientContracts::resolve(project_root.to_path_buf(), ambient)
}

fn ambient_contract_spec_from_config(
    contract: &LintCustomContractConfig,
) -> Result<AmbientContractSpec> {
    Ok(AmbientContractSpec {
        id: contract.id.clone(),
        replaces: contract.replaces.clone().unwrap_or_default(),
        when: ambient_contract_activation_from_config(&contract.when)?,
        files: contract.files.clone().unwrap_or_default(),
        effects: AmbientContractEffects {
            reads: contract.reads.clone().unwrap_or_default(),
            consumes_names: contract
                .consumes
                .as_ref()
                .and_then(|consumes| consumes.names.clone())
                .unwrap_or_default(),
            consumes_prefixes: contract
                .consumes
                .as_ref()
                .and_then(|consumes| consumes.prefixes.clone())
                .unwrap_or_default(),
            consumes_all: contract
                .consumes
                .as_ref()
                .and_then(|consumes| consumes.all)
                .unwrap_or(false),
            provides_variables: contract
                .provides
                .as_ref()
                .and_then(|provides| provides.variables.clone())
                .unwrap_or_default(),
            provides_functions: contract
                .provides
                .as_ref()
                .and_then(|provides| provides.functions.clone())
                .unwrap_or_default(),
            functions: contract
                .functions
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|function| AmbientFunctionContractSpec {
                    name: function.name,
                    reads: function.reads.unwrap_or_default(),
                    sets: function.sets.unwrap_or_default(),
                })
                .collect(),
        },
    })
}

fn ambient_contract_activation_from_config(
    when: &LintContractWhenConfig,
) -> Result<AmbientContractActivation> {
    match when {
        LintContractWhenConfig::Always(value) => {
            if value == "always" {
                Ok(AmbientContractActivation::Always)
            } else {
                Err(anyhow!(
                    "unsupported ambient contract activation {value:?}; expected \"always\" or a typed activation"
                ))
            }
        }
        LintContractWhenConfig::Activation(activation) => match activation.activation_type {
            LintContractActivationTypeConfig::Unknown => {
                Err(anyhow!("unsupported ambient contract activation type"))
            }
        },
    }
}

#[allow(dead_code)]
fn verbatim_stripped_match_path(path: &Path) -> Option<PathBuf> {
    let path = path.to_string_lossy();

    if let Some(stripped) = path.strip_prefix(r"\\?\UNC\") {
        return Some(PathBuf::from(format!(r"\\{stripped}")));
    }

    path.strip_prefix(r"\\?\").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn invalid_config() -> ShuckConfig {
        deser_toml::from_str(
            r#"
[lint]
select = ["C001", "oops"]
per-file-shell = { "a.sh" = "fish", "b.sh" = "mksh" }

[lint.rule-options.s080]
max-lines = 50
count = "bogus"
"#,
        )
        .expect("config should deserialize")
    }

    #[test]
    fn strict_policy_rejects_invalid_values() {
        let config = invalid_config();
        let error = SettingsResolver::new(Path::new("/workspace"), &config, ResolvePolicy::Strict)
            .resolve_lint()
            .unwrap_err();

        let msg = format!("{error:#}");
        assert!(
            msg.contains("invalid lint.per-file-shell shell")
                || msg.contains("invalid lint.select"),
            "{msg}"
        );
    }

    #[test]
    fn lenient_policy_skips_invalid_values_and_records_warnings() {
        let config = invalid_config();
        let mut resolver =
            SettingsResolver::new(Path::new("/workspace"), &config, ResolvePolicy::Lenient);
        let resolved = resolver.resolve_lint().unwrap();

        assert_eq!(resolved.linter.rules, LinterSettings::default_rules());
        assert_eq!(
            resolved.per_file_shell,
            vec![PerFileShell::new("b.sh", ShellDialect::Mksh)]
        );
        assert_eq!(resolved.linter.rule_options.s080.max_lines, 50);
        assert_eq!(
            resolved.linter.rule_options.s080.count,
            LinterRuleOptions::default().s080.count
        );
        assert_eq!(resolver.warnings().len(), 3, "{:?}", resolver.warnings());
    }

    #[test]
    fn config_override_layers_use_selector_specificity() {
        let config: ShuckConfig = deser_toml::from_str("[lint]\nselect = ['ALL']\n").unwrap();
        let client: ShuckConfig =
            deser_toml::from_str("[lint]\nignore = ['C']\nextend-select = ['C001']\n").unwrap();
        let overrides = [SettingsOverride::Config(Box::new(client))];
        let resolved =
            SettingsResolver::new(Path::new("/workspace"), &config, ResolvePolicy::Strict)
                .with_overrides(&overrides)
                .resolve_lint()
                .unwrap();

        assert!(
            resolved
                .linter
                .rules
                .contains(shuck_linter::Rule::UnusedAssignment)
        );
        assert!(
            !resolved
                .linter
                .rules
                .contains(shuck_linter::Rule::UndefinedVariable)
        );
        assert!(
            resolved
                .linter
                .rules
                .contains(shuck_linter::Rule::UnquotedExpansion)
        );
    }

    #[test]
    fn format_resolution_ignores_invalid_lint_selectors() {
        let config: ShuckConfig =
            deser_toml::from_str("[lint]\nselect = ['oops']\n[format]\nindent-width = 2\n")
                .unwrap();
        let resolved =
            SettingsResolver::new(Path::new("/workspace"), &config, ResolvePolicy::Strict)
                .resolve_format()
                .unwrap();

        assert_eq!(resolved.options.indent_width(), 2);
    }
}
