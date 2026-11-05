#![warn(missing_docs)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]

//! Library entrypoints for the `shuck` CLI.
//!
//! This crate primarily exists so the command-line binary, tests, and benchmarks can share the
//! same argument parsing and command execution code. Most users should invoke the `shuck` binary
//! directly rather than depend on this library API.

/// Command-line argument types and parsing helpers for the `shuck` executable.
pub mod args;
#[doc(hidden)]
pub mod cli;

mod cache;
mod color;
mod commands;
#[doc(hidden)]
pub mod config_docs;
mod discover;
mod format_settings;
mod fs_write;
#[doc(hidden)]
pub mod shellcheck_compat;
#[doc(hidden)]
pub mod shellcheck_runtime;
mod stdin;

use std::ffi::OsString;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;
use shuck_config::ConfigArguments;

pub use crate::cli::{Args, Command};
use crate::cli::format::FormatCommand;

/// Native stack for every thread that parses, analyzes, or formats scripts.
const ANALYSIS_THREAD_STACK_SIZE: usize = 64 * 1024 * 1024;

/// Exit status returned by commands.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExitStatus {
    /// The command completed successfully without reporting failures.
    Success,
    /// The command completed, but reported lint or formatting failures.
    Failure,
    /// The command failed due to invalid input, I/O, or another runtime error.
    Error,
    /// The command exited with an explicit process status code.
    Code(i32),
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => ExitCode::from(0),
            ExitStatus::Failure => ExitCode::from(1),
            ExitStatus::Error => ExitCode::from(2),
            ExitStatus::Code(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        }
    }
}

/// Run `shuck` with the provided command-line arguments and return the resulting exit code.
pub fn run<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let all_args: Vec<OsString> = args.into_iter().map(Into::into).collect();

    let _ = rayon::ThreadPoolBuilder::new()
        .stack_size(ANALYSIS_THREAD_STACK_SIZE)
        .build_global();

    let worker_args = all_args.clone();
    let worker = std::thread::Builder::new()
        .name("shuck-main".to_owned())
        .stack_size(ANALYSIS_THREAD_STACK_SIZE)
        .spawn(move || run_inner(worker_args));
    match worker {
        Ok(handle) => handle
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic)),
        Err(_) => run_inner(all_args),
    }
}

fn run_inner(argv: Vec<OsString>) -> ExitCode {
    if shellcheck_compat::should_activate(&argv) {
        return shellcheck_compat::run(argv);
    }

    let parsed = match cli::parse(argv) {
        Ok(parsed) => parsed,
        Err(err) => {
            let mut stderr = std::io::stderr().lock();
            let _ = writeln!(stderr, "{err}");
            return ExitCode::from(err.exit_code());
        }
    };

    match parsed {
        cli::ParsedCli::Help(text) => {
            print!("{text}");
            ExitCode::from(0)
        }
        cli::ParsedCli::Version(text) => {
            print!("{text}");
            ExitCode::from(0)
        }
        cli::ParsedCli::Run(args) => match cli::execute(*args) {
            Ok(ExitStatus::Code(code)) => exit_with_child_status(code),
            Ok(status) => status.into(),
            Err(err) => report_error(&err),
        },
    }
}

fn report_error(err: &anyhow::Error) -> ExitCode {
    for cause in err.chain() {
        if let Some(ioerr) = cause.downcast_ref::<std::io::Error>()
            && ioerr.kind() == std::io::ErrorKind::BrokenPipe
        {
            return ExitCode::from(0);
        }
    }

    let mut stderr = std::io::stderr().lock();
    let use_color = color::should_colorize();
    let _ = writeln!(
        stderr,
        "{}: {err}",
        color::paint("shuck", color::RED_BOLD, use_color)
    );
    for cause in err.chain().skip(1) {
        let _ = writeln!(
            stderr,
            "  {} {cause}",
            color::paint("Cause:", color::BOLD, use_color)
        );
    }

    ExitStatus::Error.into()
}

#[cfg(windows)]
fn exit_with_child_status(code: i32) -> ExitCode {
    std::process::exit(code);
}

#[cfg(not(windows))]
fn exit_with_child_status(code: i32) -> ExitCode {
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}

#[doc(hidden)]
pub fn benchmark_check_paths(
    cwd: &Path,
    paths: &[PathBuf],
    output_format: args::CheckOutputFormatArg,
) -> Result<usize> {
    commands::check::benchmark_check_paths(cwd, paths, output_format)
}

pub(crate) fn format(
    mut args: FormatCommand,
    config_arguments: &ConfigArguments,
    cache_dir: Option<&Path>,
) -> Result<ExitStatus> {
    let stdin = is_stdin(&args.files, args.stdin_filename.as_deref());
    args.files = resolve_default_files(args.files, stdin);

    if stdin {
        commands::format_stdin::format_stdin(args, config_arguments)
    } else {
        commands::format::format(args, config_arguments, cache_dir)
    }
}

fn is_stdin(files: &[PathBuf], stdin_filename: Option<&Path>) -> bool {
    if stdin_filename.is_some() {
        return true;
    }

    matches!(files, [file] if file == Path::new("-"))
}

fn resolve_default_files(files: Vec<PathBuf>, is_stdin: bool) -> Vec<PathBuf> {
    if files.is_empty() {
        if is_stdin {
            vec![PathBuf::from("-")]
        } else {
            vec![PathBuf::from(".")]
        }
    } else {
        files
    }
}
