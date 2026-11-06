use rustybench::counter::BytesCount;
use rustybench::{Bencher, black_box};
use shuck_benchmark::{
    TestCase, TestFile, benchmark_case, benchmark_case_names, configure_benchmark_allocator,
    parse_fixture,
};
use shuck_indexer::Indexer;
use shuck_linter::{
    AnalysisRequest, LinterFacts, LinterSemanticArtifacts, LinterSettings, RuleSet,
    ShellCheckCodeMap,
};
use shuck_parser::parser::ParseResult;

configure_benchmark_allocator!();

struct PreparedLinterInputs {
    files: &'static [TestFile],
    outputs: &'static [ParseResult],
    indexers: &'static [Indexer],
    semantics: &'static [LinterSemanticArtifacts<'static>],
}

fn prepare_linter_inputs(case: &'static TestCase) -> PreparedLinterInputs {
    let outputs = Box::leak(
        case.files
            .iter()
            .map(|file| parse_fixture(file.source))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let indexers = Box::leak(
        case.files
            .iter()
            .zip(outputs.iter())
            .map(|(file, output)| Indexer::new(file.source, output))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let semantics = Box::leak(
        case.files
            .iter()
            .zip(outputs.iter())
            .zip(indexers.iter())
            .map(|((file, output), indexer)| {
                LinterSemanticArtifacts::build(&output.file, file.source, indexer)
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );

    PreparedLinterInputs {
        files: case.files,
        outputs,
        indexers,
        semantics,
    }
}

fn build_linter_facts(
    source: &str,
    output: &ParseResult,
    indexer: &Indexer,
    semantic: &LinterSemanticArtifacts<'_>,
) -> usize {
    let facts = LinterFacts::build(&output.file, source, semantic, indexer);

    black_box(
        facts.commands().count()
            + facts.words().word_facts().count()
            + facts.words().single_quoted_fragments().len()
            + facts.words().backtick_fragments().len()
            + facts.words().pattern_charclass_spans().len()
            + facts.words().substring_expansion_fragments().len()
            + facts.words().case_modification_fragments().len()
            + facts.words().replacement_expansion_fragments().len(),
    )
}

fn lint_source(
    source: &str,
    settings: &LinterSettings,
    shellcheck_map: &ShellCheckCodeMap,
) -> usize {
    let output = parse_fixture(source);
    let diagnostics = AnalysisRequest::from_parse_result(&output, source, settings)
        .with_shellcheck_map(shellcheck_map)
        .lint();

    black_box(diagnostics.len())
}

#[rustybench::bench(args = benchmark_case_names())]
fn linter_facts(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    let inputs = prepare_linter_inputs(case);

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            let facts_size = inputs
                .files
                .iter()
                .zip(inputs.outputs.iter())
                .zip(inputs.indexers.iter())
                .zip(inputs.semantics.iter())
                .map(|(((file, output), indexer), semantic)| {
                    build_linter_facts(file.source, output, indexer, semantic)
                })
                .sum::<usize>();
            black_box(facts_size)
        });
}

#[rustybench::bench(args = benchmark_case_names())]
fn linter(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    let settings = LinterSettings::for_rules(RuleSet::all().iter());
    let shellcheck_map = ShellCheckCodeMap::default();

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            let diagnostic_count: usize = case
                .files
                .iter()
                .map(|file| lint_source(file.source, &settings, &shellcheck_map))
                .sum();
            black_box(diagnostic_count)
        });
}

fn main() {
    rustybench::main();
}
