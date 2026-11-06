use rustybench::{Bencher, black_box};
use shuck_benchmark::configure_benchmark_allocator;
use shuck_linter::BenchmarkCasePatternMatcher as Matcher;

configure_benchmark_allocator!();

struct PatternPair {
    name: &'static str,
    left: &'static str,
    right: &'static str,
}

const PAIRS: &[PatternPair] = &[
    // Pure literals — fixed-length fast path; no NFA work.
    PatternPair {
        name: "literal/equal",
        left: "abcdef",
        right: "abcdef",
    },
    PatternPair {
        name: "literal/diff",
        left: "abcdef",
        right: "abcxyz",
    },
    PatternPair {
        name: "literal/long_equal",
        left: "configure_benchmark_allocator",
        right: "configure_benchmark_allocator",
    },
    // Single '?' — fixed length, fast path.
    PatternPair {
        name: "qmark/single",
        left: "?",
        right: "a",
    },
    PatternPair {
        name: "qmark/triple_dot_sh",
        left: "???.sh",
        right: "foo.sh",
    },
    PatternPair {
        name: "qmark/all_qmark",
        left: "????????",
        right: "abcdefgh",
    },
    // Single-'*' simple-glob shapes — should hit PR #663's fast path.
    PatternPair {
        name: "star/bare",
        left: "*",
        right: "anything-here",
    },
    PatternPair {
        name: "star/star_vs_star",
        left: "*",
        right: "*",
    },
    PatternPair {
        name: "star/prefix",
        left: "foo*",
        right: "foobar",
    },
    PatternPair {
        name: "star/suffix",
        left: "*.txt",
        right: "readme.txt",
    },
    PatternPair {
        name: "star/middle",
        left: "foo*bar",
        right: "fooXYZbar",
    },
    PatternPair {
        name: "star/prefix_vs_prefix",
        left: "foo*",
        right: "foobar*",
    },
    PatternPair {
        name: "star/suffix_vs_suffix",
        left: "*.sh",
        right: "*.bash",
    },
    PatternPair {
        name: "star/disjoint_prefix",
        left: "foo*",
        right: "bar*",
    },
    // Multi-'*' patterns — fall through to NFA product walk.
    PatternPair {
        name: "multistar/contains",
        left: "*foo*",
        right: "abcfoodef",
    },
    PatternPair {
        name: "multistar/two_each",
        left: "*foo*",
        right: "*bar*",
    },
    PatternPair {
        name: "multistar/three_segments",
        left: "*a*b*c*",
        right: "axxbyycyy",
    },
    PatternPair {
        name: "multistar/repeated_token",
        left: "*aa*",
        right: "aaaaaaaaaa",
    },
    // Mixed '*' and '?' — NFA walk, more expensive.
    PatternPair {
        name: "mixed/qstar_around_lit",
        left: "?*foo*?",
        right: "abcfoodef",
    },
    PatternPair {
        name: "mixed/lit_q_star",
        left: "foo?*bar",
        right: "fooXbar",
    },
    PatternPair {
        name: "mixed/star_q_lit_q",
        left: "*?.t?t",
        right: "ab.txt",
    },
    PatternPair {
        name: "mixed/long_alternation",
        left: "a*?b*?c*?d",
        right: "axbycdzdz",
    },
    // Realistic shell case-arm shapes.
    PatternPair {
        name: "shell/help",
        left: "--help",
        right: "--help",
    },
    PatternPair {
        name: "shell/h_short",
        left: "-h",
        right: "--help",
    },
    PatternPair {
        name: "shell/dash_anything",
        left: "-*",
        right: "-h",
    },
    PatternPair {
        name: "shell/double_dash_anything",
        left: "--*",
        right: "--verbose",
    },
    PatternPair {
        name: "shell/sh_ext",
        left: "*.sh",
        right: "*.bash",
    },
    PatternPair {
        name: "shell/script_ext",
        left: "*.sh",
        right: "*.zsh",
    },
    PatternPair {
        name: "shell/path_under",
        left: "/usr/local/*",
        right: "/usr/local/bin/foo",
    },
    PatternPair {
        name: "shell/wildcard_anywhere",
        left: "*",
        right: "/var/log/messages",
    },
];

fn pair(name: &str) -> &'static PatternPair {
    PAIRS
        .iter()
        .find(|pair| pair.name == name)
        .expect("benchmark pair should exist")
}

fn matcher(glob: &str) -> Matcher {
    Matcher::from_glob(glob).unwrap_or_else(|| panic!("not statically analyzable: {glob}"))
}

#[rustybench::bench(args = PAIRS.iter().map(|pair| pair.name))]
fn subsumes(bencher: Bencher, pair_name: &str) {
    let pair = pair(pair_name);
    let left = matcher(pair.left);
    let right = matcher(pair.right);

    bencher.bench_local(|| {
        let a = black_box(&left).subsumes(black_box(&right));
        let b = black_box(&right).subsumes(black_box(&left));
        black_box(a) || black_box(b)
    });
}

#[rustybench::bench(args = PAIRS.iter().map(|pair| pair.name))]
fn intersects(bencher: Bencher, pair_name: &str) {
    let pair = pair(pair_name);
    let left = matcher(pair.left);
    let right = matcher(pair.right);

    bencher.bench_local(|| black_box(black_box(&left).intersects(black_box(&right))));
}

#[rustybench::bench(args = PAIRS.iter().map(|pair| pair.name))]
fn from_glob(bencher: Bencher, pair_name: &str) {
    let glob = pair(pair_name).left;
    bencher.bench_local(|| black_box(Matcher::from_glob(black_box(glob))));
}

fn main() {
    rustybench::main();
}
