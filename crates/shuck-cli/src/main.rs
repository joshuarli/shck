#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use std::process::ExitCode;

fn main() -> ExitCode {
    shuck::run(std::env::args_os())
}
