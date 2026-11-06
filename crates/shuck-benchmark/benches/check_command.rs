use std::fs;
use std::path::PathBuf;

use rustybench::counter::BytesCount;
use rustybench::{Bencher, black_box};
use shuck::args::CheckOutputFormatArg;
use shuck_benchmark::{
    TestCase, benchmark_case, benchmark_case_names, configure_benchmark_allocator,
};
use tempfile::TempDir;

configure_benchmark_allocator!();

struct PreparedCheckCase {
    _tempdir: TempDir,
    cwd: PathBuf,
    paths: Vec<PathBuf>,
}

fn prepare_check_case(case: &TestCase) -> PreparedCheckCase {
    let tempdir = match tempfile::tempdir() {
        Ok(tempdir) => tempdir,
        Err(err) => panic!("benchmark tempdir should exist: {err}"),
    };
    let cwd = tempdir.path().to_path_buf();
    let mut paths = Vec::with_capacity(case.files.len());

    for (index, file) in case.files.iter().enumerate() {
        let path = cwd.join(format!("{index:02}-{}.sh", file.name));
        if let Err(err) = fs::write(&path, file.source) {
            panic!("benchmark fixture should write: {err}");
        }
        paths.push(path);
    }

    PreparedCheckCase {
        _tempdir: tempdir,
        cwd,
        paths,
    }
}

fn bench_check_case(bencher: Bencher, case_name: &str, output_format: CheckOutputFormatArg) {
    let case = benchmark_case(case_name);
    let prepared = prepare_check_case(case);

    bencher
        .counter(BytesCount::from(case.total_bytes()))
        .bench_local(|| {
            black_box(match shuck::benchmark_check_paths(
                &prepared.cwd,
                &prepared.paths,
                output_format,
            ) {
                Ok(result) => result,
                Err(err) => panic!("check benchmark should succeed: {err}"),
            })
        });
}

#[rustybench::bench(args = benchmark_case_names())]
fn check_command_concise(bencher: Bencher, case_name: &str) {
    bench_check_case(bencher, case_name, CheckOutputFormatArg::Concise);
}

#[rustybench::bench(args = benchmark_case_names())]
fn check_command_full(bencher: Bencher, case_name: &str) {
    bench_check_case(bencher, case_name, CheckOutputFormatArg::Full);
}

fn main() {
    rustybench::main();
}
