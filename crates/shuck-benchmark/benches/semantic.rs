use rustybench::counter::{BytesCount, ItemsCount};
use rustybench::{Bencher, black_box};
use shuck_benchmark::{
    benchmark_case, benchmark_case_names, configure_benchmark_allocator, parse_fixture,
};
use shuck_indexer::Indexer;
use shuck_semantic::{ScopeKind, SemanticModel};
use std::collections::HashSet;

configure_benchmark_allocator!();

struct PreparedScopeLookupInput {
    semantic: SemanticModel,
    offsets: Vec<usize>,
}

fn build_semantic(source: &str) -> usize {
    let output = parse_fixture(source);
    let indexer = Indexer::new(source, &output);
    let semantic = SemanticModel::build(&output.file, source, &indexer);

    black_box(semantic.bindings().len() + semantic.references().len() + semantic.scopes().len())
}

fn prepare_scope_lookup_input(source: &'static str) -> PreparedScopeLookupInput {
    let output = parse_fixture(source);
    let indexer = Indexer::new(source, &output);
    let semantic = SemanticModel::build(&output.file, source, &indexer);
    let mut seen = HashSet::new();
    let mut offsets = Vec::new();

    for offset in semantic
        .scopes()
        .iter()
        .map(|scope| scope.span.start.offset())
        .chain(
            semantic
                .bindings()
                .iter()
                .map(|binding| binding.span.start.offset()),
        )
        .chain(
            semantic
                .references()
                .iter()
                .map(|reference| reference.span.start.offset()),
        )
    {
        if seen.insert(offset) {
            offsets.push(offset);
        }
    }

    if offsets.is_empty() {
        offsets.push(0);
    }

    PreparedScopeLookupInput { semantic, offsets }
}

fn run_scope_lookups(input: &PreparedScopeLookupInput) -> usize {
    let mut score = 0usize;

    for &offset in &input.offsets {
        let scope = input.semantic.scope_at(black_box(offset));
        score += match input.semantic.scope_kind(scope) {
            ScopeKind::File => 1,
            ScopeKind::Function(_) => 3,
            ScopeKind::Subshell => 5,
            ScopeKind::CommandSubstitution => 7,
            ScopeKind::Pipeline => 11,
        };
    }

    black_box(score)
}

#[rustybench::bench(args = benchmark_case_names())]
fn semantic(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            let semantic_size: usize = case
                .files
                .iter()
                .map(|file| build_semantic(file.source))
                .sum();
            black_box(semantic_size)
        });
}

#[rustybench::bench(args = benchmark_case_names())]
fn semantic_scope_at(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    let inputs = case
        .files
        .iter()
        .map(|file| prepare_scope_lookup_input(file.source))
        .collect::<Vec<_>>();
    let total_lookups: u64 = inputs.iter().map(|input| input.offsets.len() as u64).sum();

    bencher
        .counter(ItemsCount::from(total_lookups))
        .bench_local(|| {
            let score: usize = inputs.iter().map(run_scope_lookups).sum();
            black_box(score)
        });
}

fn main() {
    rustybench::main();
}
