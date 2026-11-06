use rustybench::counter::BytesCount;
use rustybench::{Bencher, black_box};
use shuck_benchmark::{
    TestCase, benchmark_case, benchmark_case_names, configure_benchmark_allocator, parse_fixture,
};
use shuck_indexer::Indexer;
use shuck_semantic::SemanticModel;

configure_benchmark_allocator!();

fn build_semantic(source: &str) -> SemanticModel {
    let output = parse_fixture(source);
    let indexer = Indexer::new(source, &output);
    SemanticModel::build(&output.file, source, &indexer)
}

fn build_semantics(case: &TestCase) -> Vec<SemanticModel> {
    case.files
        .iter()
        .map(|file| build_semantic(file.source))
        .collect()
}

#[rustybench::bench(args = benchmark_case_names())]
fn unused_assignment(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .with_inputs(|| build_semantics(case))
        .bench_local_values(|models| {
            let unused_count: usize = models
                .iter()
                .map(|model| model.analysis().unused_assignments().len())
                .sum();
            black_box(unused_count)
        });
}

#[rustybench::bench(args = benchmark_case_names())]
fn uninitialized_reference(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .with_inputs(|| build_semantics(case))
        .bench_local_values(|models| {
            let reference_count: usize = models
                .iter()
                .map(|model| model.analysis().uninitialized_references().len())
                .sum();
            black_box(reference_count)
        });
}

#[rustybench::bench(args = benchmark_case_names())]
fn variable_dataflow_combined(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .with_inputs(|| build_semantics(case))
        .bench_local_values(|models| {
            let combined_count: usize = models
                .iter()
                .map(|model| {
                    let analysis = model.analysis();
                    analysis.unused_assignments().len()
                        + analysis.uninitialized_references().len()
                })
                .sum();
            black_box(combined_count)
        });
}

#[rustybench::bench(args = benchmark_case_names())]
fn dead_code(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .with_inputs(|| build_semantics(case))
        .bench_local_values(|models| {
            let dead_code_count: usize = models
                .iter()
                .map(|model| model.analysis().dead_code().len())
                .sum();
            black_box(dead_code_count)
        });
}

fn main() {
    rustybench::main();
}
