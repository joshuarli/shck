use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=SHUCK_GIT_SHA");

    let sha = if let Ok(override_sha) = env::var("SHUCK_GIT_SHA") {
        let trimmed = override_sha.trim().to_string();
        if trimmed.is_empty() {
            panic!("SHUCK_GIT_SHA override cannot be empty");
        }
        validate_sha(&trimmed);
        trimmed
    } else {
        match resolve_git_dir() {
            Some(git_dir) => {
                watch_git_dir(&git_dir);
                let git_sha = run_git(&["rev-parse", "--short", "HEAD"]);
                validate_sha(&git_sha);
                git_sha
            }
            None => {
                panic!(
                    "no git repository found; release archives and builds without a git checkout must provide the commit SHA via the SHUCK_GIT_SHA environment variable"
                );
            }
        }
    };

    println!("cargo:rustc-env=SHUCK_GIT_SHA={sha}");
}

fn validate_sha(sha: &str) {
    if sha.is_empty() {
        panic!("git SHA cannot be empty");
    }
    if sha.eq_ignore_ascii_case("unknown") || sha.contains("dirty") {
        panic!("git SHA cannot be unknown or dirty: `{sha}`");
    }
    if !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        panic!("git SHA must consist of hexadecimal characters: `{sha}`");
    }
}

fn resolve_git_dir() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path_str = String::from_utf8(output.stdout).ok()?;
    let path = PathBuf::from(path_str.trim());
    if path.exists() { Some(path) } else { None }
}

fn watch_git_dir(git_dir: &Path) {
    let head_path = git_dir.join("HEAD");
    if head_path.exists() {
        println!("cargo:rerun-if-changed={}", head_path.display());
        if let Ok(head_content) = fs::read_to_string(&head_path) {
            let trimmed = head_content.trim();
            if let Some(ref_path) = trimmed.strip_prefix("ref: ") {
                let resolved = git_dir.join(ref_path.trim());
                if resolved.exists() {
                    println!("cargo:rerun-if-changed={}", resolved.display());
                }
            }
        }
    }
    let packed_refs = git_dir.join("packed-refs");
    if packed_refs.exists() {
        println!("cargo:rerun-if-changed={}", packed_refs.display());
    }
}

fn run_git(args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .output()
        .expect("failed to execute git command");
    if !output.status.success() {
        panic!("git command failed: git {}", args.join(" "));
    }
    String::from_utf8(output.stdout)
        .expect("git output must be UTF-8")
        .trim()
        .to_string()
}
