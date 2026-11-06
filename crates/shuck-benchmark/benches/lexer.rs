use rustybench::counter::BytesCount;
use rustybench::{Bencher, black_box};
use shuck_benchmark::{
    benchmark_case, benchmark_case_names, configure_benchmark_allocator,
};
use shuck_parser::parser::Lexer;

configure_benchmark_allocator!();

fn lex_source(source: &str) -> usize {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    while let Some(token) = lexer.next_lexed_token() {
        tokens.push(token);
    }

    black_box(tokens.len())
}

#[rustybench::bench(args = benchmark_case_names())]
fn lex(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            let token_count: usize = case.files.iter().map(|file| lex_source(file.source)).sum();
            black_box(token_count)
        });
}

fn main() {
    rustybench::main();
}
