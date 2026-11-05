pub const SHORT_SHA: &str = env!("SHUCK_GIT_SHA");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn version_string() -> String {
    format!("shuck {VERSION} (git {SHORT_SHA})\n")
}
