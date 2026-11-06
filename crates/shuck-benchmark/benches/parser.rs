use rustybench::counter::BytesCount;
use rustybench::{Bencher, black_box};
use shuck_benchmark::{
    benchmark_case, benchmark_case_names, configure_benchmark_allocator, parse_fixture,
};

configure_benchmark_allocator!();

fn parse_source(source: &str) -> usize {
    let output = parse_fixture(source);
    black_box(output.file.body.len())
}

#[rustybench::bench(args = benchmark_case_names())]
fn parse(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            let command_count: usize = case
                .files
                .iter()
                .map(|file| parse_source(file.source))
                .sum();
            black_box(command_count)
        });
}

fn main() {
    rustybench::main();
}
