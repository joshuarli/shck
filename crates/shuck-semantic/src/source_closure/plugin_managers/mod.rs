use std::path::{Path, PathBuf};

use rustc_hash::FxHashSet;
use shuck_ast::File;

use super::*;
use crate::{Name, PluginRequest, ScopeId, SemanticModel, SyntheticRead};






#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct AstFacts;

#[allow(dead_code)]
pub(super) struct PluginManagerContext<'a> {
    pub(super) model: &'a SemanticModel,
    pub(super) file: &'a File,
    pub(super) source: &'a str,
    pub(super) source_path: &'a Path,
    pub(super) plugin_resolver: &'a (dyn PluginResolver + Send + Sync),
}

#[allow(dead_code)]
pub(super) struct DeferredPluginRuntimeContext<'a> {
    pub(super) semantic: &'a SemanticModel,
    pub(super) analysis: &'a crate::SemanticAnalysis<'a>,
    pub(super) facts: &'a super::AstFacts,
    pub(super) source: &'a str,
    pub(super) scope: ScopeId,
    pub(super) synthetic_reads: &'a [SyntheticRead],
}



pub(super) fn collect_plugin_requests(
    _model: &SemanticModel,
    _file: &File,
    _source: &str,
    _source_path: &Path,
    _plugin_resolver: &(dyn crate::PluginResolver + Send + Sync),
) -> Vec<crate::PluginRequest> {
    Vec::new()
}

pub(super) fn deferred_zsh_entrypoint_required_reads(
    _semantic: &SemanticModel,
    _analysis: &crate::SemanticAnalysis<'_>,
    _facts: &super::AstFacts,
    _source: &str,
    _scope: ScopeId,
    _synthetic_reads: &[SyntheticRead],
) -> Vec<Name> {
    Vec::new()
}

#[allow(dead_code)]
pub(super) fn dedup_plugin_requests(requests: Vec<PluginRequest>) -> Vec<crate::PluginRequest> {
    let mut seen = FxHashSet::default();
    let mut out = Vec::new();
    for request in requests {
        let key = (
            request.framework.clone(),
            request.kind,
            request.name.clone(),
            request.span.start.offset(),
            request.root_hint.clone(),
        );
        if seen.insert(key) {
            out.push(request);
        }
    }
    out
}

pub(super) fn sorted_dependency_paths(_paths: Vec<PathBuf>) -> Vec<PathBuf> {
    Vec::new()
}


#[allow(dead_code)]
pub fn zsh_plugin_frameworks() -> Vec<()> { Vec::new() }
#[allow(dead_code)]
pub fn layout_for_plugin_framework(_f: &()) -> Option<()> { None }
