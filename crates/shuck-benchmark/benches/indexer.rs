use rustybench::counter::BytesCount;
use rustybench::{Bencher, black_box};
use shuck_benchmark::{
    benchmark_case, benchmark_case_names, configure_benchmark_allocator, parse_fixture,
};
use shuck_indexer::Indexer;
use shuck_parser::parser::ParseResult;

configure_benchmark_allocator!();

struct PreparedInput {
    source: &'static str,
    output: ParseResult,
}

fn prepare_input(source: &'static str) -> PreparedInput {
    let output = parse_fixture(source);
    PreparedInput { source, output }
}

fn build_indexer(input: &PreparedInput) -> usize {
    let indexer = Indexer::new(input.source, &input.output);
    black_box(
        indexer.line_index().line_count()
            + indexer.comment_index().comments().len()
            + indexer.continuation_line_starts().len(),
    )
}

#[rustybench::bench(args = benchmark_case_names())]
fn index(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    let inputs = case
        .files
        .iter()
        .map(|file| prepare_input(file.source))
        .collect::<Vec<_>>();

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            let indexed_size: usize = inputs.iter().map(build_indexer).sum();
            black_box(indexed_size)
        });
}

fn main() {
    rustybench::main();
}
