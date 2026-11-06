use rustybench::counter::BytesCount;
use rustybench::{Bencher, black_box};
use shuck_benchmark::configure_benchmark_allocator;
use shuck_parser::parser::Parser;

configure_benchmark_allocator!();

const ARITHMETIC_CASE_NAMES: &[&str] = &[
    "double-paren-command",
    "arithmetic-expansion",
    "subscripts-and-slices",
    "arithmetic-for",
    "shell-word-operands",
];

#[derive(Debug, Clone)]
struct ArithmeticCase {
    name: &'static str,
    source: String,
}

impl ArithmeticCase {
    fn total_bytes(&self) -> u64 {
        self.source.len() as u64
    }
}

fn arithmetic_case(name: &'static str, snippet: &str, repetitions: usize) -> ArithmeticCase {
    ArithmeticCase {
        name,
        source: repeat_snippet(snippet, repetitions),
    }
}

fn repeat_snippet(snippet: &str, repetitions: usize) -> String {
    let mut source = String::with_capacity(snippet.len() * repetitions);
    for _ in 0..repetitions {
        source.push_str(snippet);
    }
    source
}

fn arithmetic_cases() -> Vec<ArithmeticCase> {
    vec![
        arithmetic_case(
            "double-paren-command",
            "(( ++i ? j-- : (k = 1), m ))\n",
            400,
        ),
        arithmetic_case(
            "arithmetic-expansion",
            "echo \"$((1 + 2))\" \"$[3 + 4]\"\n",
            350,
        ),
        arithmetic_case(
            "subscripts-and-slices",
            "a[i + 1]=x\ndeclare foo[1+2]\necho ${arr[i+1]} ${s:i+1:len*2} ${arr[@]:i:j}\n",
            250,
        ),
        arithmetic_case(
            "arithmetic-for",
            "for (( i = 0 ; i < 10 ; i += ($# - 1))); do echo \"$i\"; done\n",
            180,
        ),
        arithmetic_case("shell-word-operands", "(( \"$(date -u)\" + '3' ))\n", 300),
    ]
}

fn parse_source(source: &str) -> usize {
    let output = Parser::new(black_box(source)).parse();
    if output.is_err() {
        panic!(
            "arithmetic benchmark inputs should parse: {}",
            output.strict_error()
        );
    }

    black_box(output.file.body.len())
}

#[rustybench::bench(args = ARITHMETIC_CASE_NAMES, sample_count = 50)]
fn arithmetic(bencher: Bencher, case_name: &str) {
    let case = arithmetic_cases()
        .into_iter()
        .find(|case| case.name == case_name)
        .expect("benchmark case should exist");

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| parse_source(black_box(case.source.as_str())));
}

fn main() {
    rustybench::main();
}
