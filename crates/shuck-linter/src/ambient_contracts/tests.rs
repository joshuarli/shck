use std::path::Path;

use shuck_indexer::Indexer;
use shuck_parser::parser::Parser;
use shuck_semantic::{
    FileContract, FileEntryContractCollector, NoopTraversalObserver, PluginFramework,
    PluginRequest, PluginRequestKind, SemanticBuildOptions, build_with_observer_with_options,
};
use std::sync::Arc;

use super::{AmbientContractCollector, AmbientContractConfig, ResolvedAmbientContracts};
use crate::ShellDialect;

fn contract_for(path: &Path, source: &str) -> Option<FileContract> {
    contract_for_shell(path, source, ShellDialect::Sh)
}

fn contract_for_shell(path: &Path, source: &str, shell: ShellDialect) -> Option<FileContract> {
    contract_for_optional_path(Some(path), source, shell)
}

fn contract_for_optional_path(
    path: Option<&Path>,
    source: &str,
    shell: ShellDialect,
) -> Option<FileContract> {
    contract_for_optional_path_with_contracts(
        path,
        source,
        shell,
        Arc::new(ResolvedAmbientContracts::default()),
    )
}

fn contract_for_optional_path_with_contracts(
    path: Option<&Path>,
    source: &str,
    shell: ShellDialect,
    contracts: Arc<ResolvedAmbientContracts>,
) -> Option<FileContract> {
    let output = Parser::new(source).parse().unwrap();
    let indexer = Indexer::new(source, &output);
    let mut observer = NoopTraversalObserver;
    let mut collector = AmbientContractCollector::new(source, path, shell, contracts);
    let _semantic = build_with_observer_with_options(
        &output.file,
        source,
        &indexer,
        &mut observer,
        SemanticBuildOptions {
            file_entry_contract_collector: Some(&mut collector),
            ..SemanticBuildOptions::default()
        },
    );
    collector.finish()
}

fn request_contract_for(path: &Path, request: PluginRequest) -> FileContract {
    ResolvedAmbientContracts::default()
        .request_contracts_for_plugin(path, &request)
        .requesting_file_contract
}

fn has_initialized_binding(contract: &FileContract, name: &str) -> bool {
    contract.provided_bindings.iter().any(|binding| {
        binding.name == name
            && binding.file_entry_initialization
                == shuck_semantic::FileEntryBindingInitialization::Initialized
    })
}

fn has_ambient_binding(contract: &FileContract, name: &str) -> bool {
    contract.provided_bindings.iter().any(|binding| {
        binding.name == name
            && binding.file_entry_initialization
                == shuck_semantic::FileEntryBindingInitialization::AmbientOnly
    })
}

fn has_consumed_prefix(contract: &FileContract, prefix: &str) -> bool {
    contract
        .externally_consumed_binding_prefixes
        .iter()
        .any(|consumed_prefix| consumed_prefix.as_str() == prefix)
}

fn has_consumed_name(contract: &FileContract, name: &str) -> bool {
    contract
        .externally_consumed_binding_names
        .iter()
        .any(|consumed_name| consumed_name.as_str() == name)
}

fn plugin_request(framework: PluginFramework, name: &str) -> PluginRequest {
    PluginRequest {
        framework,
        kind: PluginRequestKind::Plugin,
        name: name.to_owned(),
        span: shuck_ast::Span::new(),
        explicit: false,
        root_hint: None,
    }
}

#[test]
fn project_specific_paths_do_not_get_ambient_contracts() {
    let void_path = Path::new("/tmp/void-packages/common/build-style/void-cross.sh");
    let void_source = "\
helper() { cd \"${wrksrc}\"; }
printf '%s\\n' \"$pkgname\" \"$pkgver\" \"$XBPS_SRCPKGDIR\" \"$configure_args\"
";
    assert!(contract_for(void_path, void_source).is_none());

    let flattened_path =
        Path::new("/tmp/scripts/void-linux__void-packages__common__build-style__void-cross.sh");
    let flattened_source = "\
helper() { :; }
printf '%s\\n' \"$XBPS_SRCPKGDIR\" \"$configure_args\" \"$wrksrc\"
";
    assert!(contract_for(flattened_path, flattened_source).is_none());
}

#[test]
fn bash_it_theme_paths_get_palette_ambient_contracts() {
    let path = Path::new("/tmp/Bash-it/themes/example/example.theme.bash");
    let source = "\
prompt_command() {
  PS1=\"${green?} ${green} ${reset_color?}\"
}
PROMPT_COMMAND=prompt_command
";

    let contract = contract_for(path, source).unwrap();

    assert!(has_ambient_binding(&contract, "green"));
    assert!(has_ambient_binding(&contract, "reset_color"));
    assert!(!has_initialized_binding(&contract, "green"));
    assert!(!has_initialized_binding(&contract, "reset_color"));
    assert!(!contract.externally_consumed_bindings);
}

#[test]
fn disabling_bash_it_theme_selector_removes_palette_bindings() {
    let path = Path::new("/tmp/Bash-it/themes/example/example.theme.bash");
    let source = "\
prompt_command() {
  PS1=\"${green?} ${green} ${reset_color?}\"
}
PROMPT_COMMAND=prompt_command
";
    let contracts = Arc::new(
        ResolvedAmbientContracts::resolve(
            "/tmp",
            AmbientContractConfig {
                disabled: vec!["bash-it/theme".to_owned()],
                ..AmbientContractConfig::default()
            },
        )
        .unwrap(),
    );

    let contract =
        contract_for_optional_path_with_contracts(Some(path), source, ShellDialect::Sh, contracts);

    assert!(contract.is_none(), "{contract:?}");
}

#[test]
fn generic_theme_paths_do_not_initialize_palette_contracts() {
    let path = Path::new("/tmp/project/themes/example.theme.bash");
    let source = "\
helper() {
  printf '%s\\n' \"$green\" \"$reset_color\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn generic_completion_paths_do_not_initialize_completion_contracts() {
    let path = Path::new("/tmp/project/completions/example.sh");
    let source = "\
helper() {
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn generic_completion_paths_with_compreply_do_not_initialize_completion_contracts() {
    let path = Path::new("/tmp/project/completions/example.sh");
    let source = "\
helper() {
  COMPREPLY=()
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn bash_completion_paths_without_initializer_do_not_initialize_completion_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
_example() {
  COMPREPLY=()
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn bash_completion_paths_with_initializer_get_ambient_completion_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
_example() {
  _init_completion || return
  printf '%s\\n' \"$cur\" \"$cword\" \"$comp_args\"
}
";

    let contract = contract_for(path, source).unwrap();

    assert!(has_ambient_binding(&contract, "cur"));
    assert!(has_ambient_binding(&contract, "cword"));
    assert!(has_ambient_binding(&contract, "comp_args"));
    assert!(!has_initialized_binding(&contract, "cur"));
    assert!(!has_initialized_binding(&contract, "cword"));
    assert!(!has_initialized_binding(&contract, "comp_args"));
    assert!(!contract.externally_consumed_bindings);
}

#[test]
fn bash_completion_paths_with_chained_initializer_wrappers_get_ambient_completion_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
_example() {
  command env LC_ALL=C _init_completion || return
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    let contract = contract_for(path, source).unwrap();

    assert!(has_ambient_binding(&contract, "cur"));
    assert!(has_ambient_binding(&contract, "cword"));
    assert!(!has_initialized_binding(&contract, "cur"));
    assert!(!has_initialized_binding(&contract, "cword"));
    assert!(!contract.externally_consumed_bindings);
}

#[test]
fn bash_completion_paths_with_env_then_shell_wrapper_get_ambient_completion_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
_example() {
  env LC_ALL=C command _init_completion || return
  env LC_ALL=\"$locale\" _get_comp_words_by_ref cur || return
  env LC_ALL=C env _comp_initialize || return
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    let contract = contract_for(path, source).unwrap();

    assert!(has_ambient_binding(&contract, "cur"));
    assert!(has_ambient_binding(&contract, "cword"));
    assert!(!has_initialized_binding(&contract, "cur"));
    assert!(!has_initialized_binding(&contract, "cword"));
    assert!(!contract.externally_consumed_bindings);
}

#[test]
fn bash_completion_paths_with_external_initializer_wrappers_do_not_initialize_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
_example() {
  sudo _init_completion || return
  sudo env _init_completion || return
  env LC_ALL=C sudo _get_comp_words_by_ref || return
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn bash_completion_paths_with_subshell_initializer_do_not_initialize_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
_example() {
  printf '%s\\n' \"$(_init_completion)\" \"$cur\" \"$cword\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn bash_completion_paths_with_commented_initializer_do_not_initialize_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
# TODO: call _init_completion later
_example() {
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn bash_completion_paths_with_wrapper_identifier_do_not_initialize_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
_example() {
  my_init_completion_wrapper || return
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn bash_completion_paths_with_initializer_definition_do_not_initialize_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
_init_completion() {
  :
}
_example() {
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn bash_completion_paths_with_separator_comment_do_not_initialize_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
noop;# _init_completion later
_example() {
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn bash_completion_paths_with_initializer_in_heredoc_do_not_initialize_contracts() {
    let path = Path::new("/tmp/bash-completion/completions/example.bash");
    let source = "\
cat <<EOF
_init_completion
EOF
_example() {
  printf '%s\\n' \"$cur\" \"$cword\"
}
";

    assert!(contract_for(path, source).is_none());
}

#[test]
fn sourced_runtime_module_paths_do_not_initialize_arbitrary_reads() {
    let path = Path::new("/tmp/LinuxGSM/lgsm/modules/command_backup.sh");
    let source = "\
commandname=\"BACKUP\"
backup_run() {
  printf '%s\\n' \"$lockdir\" \"$commandname\"
}
";

    assert!(contract_for(path, source).is_none());
}
