use std::io::{self, IsTerminal};
use std::sync::atomic::{AtomicU8, Ordering};

const AUTO: u8 = 0;
const ALWAYS: u8 = 1;
const NEVER: u8 = 2;

static COLOR_OVERRIDE: AtomicU8 = AtomicU8::new(AUTO);

pub(crate) const RESET: &str = "\x1b[0m";
pub(crate) const BOLD: &str = "\x1b[1m";
pub(crate) const BOLD_UNDERLINE: &str = "\x1b[1;4m";
pub(crate) const RED: &str = "\x1b[31m";
pub(crate) const GREEN: &str = "\x1b[32m";
pub(crate) const YELLOW: &str = "\x1b[33m";
pub(crate) const CYAN: &str = "\x1b[36m";
pub(crate) const RED_BOLD: &str = "\x1b[1;31m";
pub(crate) const YELLOW_BOLD: &str = "\x1b[1;33m";
pub(crate) const BLUE_BOLD: &str = "\x1b[1;34m";
pub(crate) const CYAN_BOLD: &str = "\x1b[1;36m";

pub(crate) fn set_override(enabled: bool) {
    COLOR_OVERRIDE.store(if enabled { ALWAYS } else { NEVER }, Ordering::Relaxed);
}

#[cfg(test)]
pub(crate) fn unset_override() {
    COLOR_OVERRIDE.store(AUTO, Ordering::Relaxed);
}

pub(crate) fn should_colorize() -> bool {
    match COLOR_OVERRIDE.load(Ordering::Relaxed) {
        ALWAYS => true,
        NEVER => false,
        _ => auto_detect(),
    }
}

fn auto_detect() -> bool {
    if let Ok(force) = std::env::var("FORCE_COLOR")
        && !force.is_empty()
        && force != "0"
    {
        return true;
    }
    if let Ok(no_color) = std::env::var("NO_COLOR")
        && !no_color.is_empty()
    {
        return false;
    }
    if let Ok(clicolor) = std::env::var("CLICOLOR")
        && clicolor == "0"
    {
        return false;
    }
    if let Ok(clicolor_force) = std::env::var("CLICOLOR_FORCE")
        && clicolor_force != "0"
        && !clicolor_force.is_empty()
    {
        return true;
    }
    if let Ok(term) = std::env::var("TERM")
        && term == "dumb"
    {
        return false;
    }
    io::stdout().is_terminal()
}

pub(crate) fn paint(value: impl std::fmt::Display, style: &str, use_color: bool) -> String {
    if use_color {
        format!("{style}{value}{RESET}")
    } else {
        value.to_string()
    }
}
