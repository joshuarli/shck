use rustybench::counter::BytesCount;
use rustybench::{Bencher, black_box};
use shuck_benchmark::{
    benchmark_case, benchmark_case_names, configure_benchmark_allocator,
};
use shuck_parser::parser::Lexer;

configure_benchmark_allocator!();

fn lex_source_hot_path(source: &str) -> usize {
    let mut lexer = Lexer::new(source);
    let mut token_count = 0usize;

    while let Some(_kind) = lexer.next_token_kind() {
        token_count += 1;
    }

    black_box(token_count)
}

#[rustybench::bench(args = benchmark_case_names())]
fn lex_hot_path(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            let token_count: usize = case
                .files
                .iter()
                .map(|file| lex_source_hot_path(file.source))
                .sum();
            black_box(token_count)
        });
}

fn main() {
    rustybench::main();
}
