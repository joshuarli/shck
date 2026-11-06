use rustybench::counter::BytesCount;
use rustybench::{Bencher, black_box};
use shuck_benchmark::{
    benchmark_case, benchmark_case_names, configure_benchmark_allocator, parse_fixture,
};
use shuck_formatter::{
    FormattedSource, ShellFormatOptions, build_formatter_facts, format_file_ast, format_source,
};

configure_benchmark_allocator!();

fn format_source_bytes(source: &str, options: &ShellFormatOptions) -> usize {
    let formatted = match format_source(black_box(source), None, options) {
        Ok(formatted) => formatted,
        Err(err) => panic!("formatter benchmark inputs should format: {err}"),
    };

    output_bytes(source, formatted)
}

fn format_file_ast_bytes(
    source: &str,
    parsed: shuck_parser::parser::ParseResult,
    options: &ShellFormatOptions,
) -> usize {
    let formatted = match format_file_ast(black_box(source), black_box(parsed.file), None, options)
    {
        Ok(formatted) => formatted,
        Err(err) => panic!("formatter AST benchmark inputs should format: {err}"),
    };

    output_bytes(source, formatted)
}

fn formatter_fact_items(source: &str, parsed: &shuck_parser::parser::ParseResult) -> usize {
    black_box(build_formatter_facts(
        black_box(source),
        black_box(&parsed.file),
    ))
}

fn output_bytes(source: &str, formatted: FormattedSource) -> usize {
    match formatted {
        FormattedSource::Unchanged => black_box(source.len()),
        FormattedSource::Formatted(formatted) => black_box(formatted.len()),
    }
}

#[rustybench::bench(args = benchmark_case_names())]
fn formatter_source(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    let options = ShellFormatOptions::default();

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            let output_bytes: usize = case
                .files
                .iter()
                .map(|file| format_source_bytes(file.source, &options))
                .sum();
            black_box(output_bytes)
        });
}

#[rustybench::bench(args = benchmark_case_names())]
fn formatter_ast(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    let options = ShellFormatOptions::default();
    let parsed_files = case
        .files
        .iter()
        .map(|file| parse_fixture(file.source))
        .collect::<Vec<_>>();

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .with_inputs(|| parsed_files.clone())
        .bench_local_values(|parsed_files| {
            let output_bytes: usize = case
                .files
                .iter()
                .zip(parsed_files)
                .map(|(file, parsed)| format_file_ast_bytes(file.source, parsed, &options))
                .sum();
            black_box(output_bytes)
        });
}

#[rustybench::bench(args = benchmark_case_names())]
fn formatter_facts(bencher: Bencher, case_name: &str) {
    let case = benchmark_case(case_name);
    let parsed_files = case
        .files
        .iter()
        .map(|file| parse_fixture(file.source))
        .collect::<Vec<_>>();

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            let comment_items: usize = case
                .files
                .iter()
                .zip(parsed_files.iter())
                .map(|(file, parsed)| formatter_fact_items(file.source, parsed))
                .sum();
            black_box(comment_items)
        });
}

fn main() {
    rustybench::main();
}
