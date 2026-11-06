//! Minimal absolute URL handling for registry documents and file paths.
//!
//! Shuck needs only the URL forms it uses itself: absolute `https` and `file`
//! URLs, references resolved against a base document, and conversion between
//! file URLs and filesystem paths.

use std::fmt;
use std::path::{Path, PathBuf};

/// Error returned when a URL cannot be parsed or converted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlError {
    message: String,
}

impl UrlError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for UrlError {}

/// An absolute URL with a scheme and authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    scheme: String,
    authority: String,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl Url {
    /// Parse an absolute URL such as `https://example.com/path` or `file:///a/b`.
    pub fn parse(input: &str) -> Result<Self, UrlError> {
        let (scheme, rest) = split_scheme(input)
            .ok_or_else(|| UrlError::new(format!("`{input}` is not an absolute URL")))?;
        let rest = rest
            .strip_prefix("//")
            .ok_or_else(|| UrlError::new(format!("`{input}` must include a `//` authority")))?;
        Self::from_parts(scheme, rest)
    }

    /// Build a file URL from an absolute filesystem path.
    pub fn from_file_path(path: impl AsRef<Path>) -> Result<Self, UrlError> {
        let path = path.as_ref();
        if !path.is_absolute() {
            return Err(UrlError::new(format!(
                "`{}` is not an absolute path",
                path.display()
            )));
        }

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;

            Ok(Self {
                scheme: String::from("file"),
                authority: String::new(),
                path: encode_path_bytes(path.as_os_str().as_bytes()),
                query: None,
                fragment: None,
            })
        }

        #[cfg(not(unix))]
        {
            let normalized = path.to_string_lossy().replace('\\', "/");
            let (authority, path) = match normalized.strip_prefix("//") {
                Some(rest) => match rest.find('/') {
                    Some(index) => (&rest[..index], &rest[index..]),
                    None => (rest, ""),
                },
                None => ("", normalized.as_str()),
            };
            let path = if path.starts_with('/') {
                path.to_owned()
            } else {
                format!("/{path}")
            };

            Ok(Self {
                scheme: String::from("file"),
                authority: authority.to_owned(),
                path: encode_path_bytes(path.as_bytes()),
                query: None,
                fragment: None,
            })
        }
    }

    /// Build a file URL from a directory path, always ending in `/`.
    pub fn from_directory_path(path: impl AsRef<Path>) -> Result<Self, UrlError> {
        let mut url = Self::from_file_path(path)?;
        if !url.path.ends_with('/') {
            url.path.push('/');
        }
        Ok(url)
    }

    /// Return the URL scheme.
    #[must_use]
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// Return the URL path component.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Resolve a reference against this URL.
    pub fn join(&self, reference: &str) -> Result<Self, UrlError> {
        if reference.is_empty() {
            return Ok(self.clone());
        }

        if has_scheme(reference) {
            return Self::parse(reference);
        }

        if reference.starts_with("//") {
            let mut absolute = String::with_capacity(self.scheme.len() + 1 + reference.len());
            absolute.push_str(&self.scheme);
            absolute.push(':');
            absolute.push_str(reference);
            return Self::parse(&absolute);
        }

        let (rest, fragment) = split_once(reference, '#');
        let (reference_path, query) = split_once(rest, '?');

        let path = if reference_path.is_empty() {
            self.path.clone()
        } else if reference_path.starts_with('/') {
            remove_dot_segments(reference_path)
        } else {
            let base = match self.path.rfind('/') {
                Some(index) => &self.path[..=index],
                None => "/",
            };
            remove_dot_segments(&format!("{base}{reference_path}"))
        };

        Ok(Self {
            scheme: self.scheme.clone(),
            authority: self.authority.clone(),
            path,
            query: match query {
                Some(query) => Some(query.to_owned()),
                None if reference_path.is_empty() => self.query.clone(),
                None => None,
            },
            fragment: fragment.map(str::to_owned),
        })
    }

    /// Convert a file URL to a filesystem path.
    pub fn to_file_path(&self) -> Result<PathBuf, UrlError> {
        if self.scheme != "file" {
            return Err(UrlError::new(format!(
                "`{self}` is not a file URL"
            )));
        }
        if !self.authority.is_empty() && !self.authority.eq_ignore_ascii_case("localhost") {
            return Err(UrlError::new(format!(
                "`{self}` refers to a non-local host"
            )));
        }

        let bytes = decode_path_bytes(&self.path)?;

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;

            if !bytes.starts_with(b"/") {
                return Err(UrlError::new(format!("`{self}` is not an absolute file URL")));
            }
            Ok(PathBuf::from(std::ffi::OsString::from_vec(bytes)))
        }

        #[cfg(not(unix))]
        {
            let decoded = String::from_utf8(bytes)
                .map_err(|_| UrlError::new(format!("`{self}` is not valid UTF-8")))?;
            let path = decoded.strip_prefix('/').unwrap_or(&decoded).replace('/', "\\");
            if path.is_empty() {
                return Err(UrlError::new(format!("`{self}` is not an absolute file URL")));
            }
            Ok(PathBuf::from(path))
        }
    }

    fn from_parts(scheme: String, rest: &str) -> Result<Self, UrlError> {
        let (rest, fragment) = split_once(rest, '#');
        let (rest, query) = split_once(rest, '?');
        let (authority, path) = match rest.find('/') {
            Some(index) => (&rest[..index], &rest[index..]),
            None => (rest, ""),
        };
        let path = if path.is_empty() {
            String::from("/")
        } else {
            path.to_owned()
        };

        Ok(Self {
            scheme,
            authority: authority.to_owned(),
            path,
            query: query.map(str::to_owned),
            fragment: fragment.map(str::to_owned),
        })
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}", self.scheme, self.authority)?;
        f.write_str(&self.path)?;
        if let Some(query) = &self.query {
            write!(f, "?{query}")?;
        }
        if let Some(fragment) = &self.fragment {
            write!(f, "#{fragment}")?;
        }
        Ok(())
    }
}

fn split_scheme(input: &str) -> Option<(String, &str)> {
    let index = input.find(':')?;
    let scheme = &input[..index];
    if scheme.is_empty() || !scheme.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
        return None;
    }
    if !scheme
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
    {
        return None;
    }
    Some((scheme.to_ascii_lowercase(), &input[index + 1..]))
}

fn has_scheme(input: &str) -> bool {
    split_scheme(input).is_some()
}

fn split_once(input: &str, delimiter: char) -> (&str, Option<&str>) {
    match input.find(delimiter) {
        Some(index) => (&input[..index], Some(&input[index + delimiter.len_utf8()..])),
        None => (input, None),
    }
}

fn remove_dot_segments(path: &str) -> String {
    let absolute = path.starts_with('/');
    let trailing_slash = path.ends_with('/');
    let mut segments = Vec::new();

    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            segment => segments.push(segment),
        }
    }

    let mut result = String::new();
    if absolute {
        result.push('/');
    }
    result.push_str(&segments.join("/"));
    if trailing_slash && !result.ends_with('/') {
        result.push('/');
    }
    result
}

fn encode_path_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len());
    for &byte in bytes {
        if is_path_safe(byte) {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[(byte >> 4) as usize]));
            encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
        }
    }
    encoded
}

fn is_path_safe(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'-' | b'.' | b'_' | b'~' | b'/' | b':' | b'!' | b'$' | b'&' | b'\'' | b'(' | b')'
                | b'*' | b'+' | b',' | b';' | b'=' | b'@'
        )
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

fn decode_path_bytes(path: &str) -> Result<Vec<u8>, UrlError> {
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let Some(high) = bytes.get(index + 1).and_then(|byte| hex_value(*byte)) else {
                return Err(UrlError::new(format!(
                    "invalid percent escape in `{path}`"
                )));
            };
            let Some(low) = bytes.get(index + 2).and_then(|byte| hex_value(*byte)) else {
                return Err(UrlError::new(format!(
                    "invalid percent escape in `{path}`"
                )));
            };
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    Ok(decoded)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_https_urls() {
        let url = Url::parse("https://example.com/shells").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.path(), "/shells");
        assert_eq!(url.to_string(), "https://example.com/shells");
    }

    #[test]
    fn resolves_relative_references() {
        let base = Url::parse("https://example.com/registry/index.json").unwrap();
        assert_eq!(
            base.join("bash.json").unwrap().to_string(),
            "https://example.com/registry/bash.json"
        );
        assert_eq!(
            base.join("../other.json").unwrap().to_string(),
            "https://example.com/other.json"
        );
        assert_eq!(
            base.join("https://cdn.example.com/archive.tar.gz")
                .unwrap()
                .to_string(),
            "https://cdn.example.com/archive.tar.gz"
        );
        assert_eq!(
            base.join("/root.json").unwrap().to_string(),
            "https://example.com/root.json"
        );
    }

    #[test]
    fn resolves_directory_base() {
        let base = Url::parse("https://example.com/shuck-shells/").unwrap();
        assert_eq!(base.path(), "/shuck-shells/");
        assert_eq!(
            base.join("index.json").unwrap().to_string(),
            "https://example.com/shuck-shells/index.json"
        );
    }

    #[test]
    fn converts_file_paths_to_urls() {
        let url = Url::from_file_path("/tmp/shuck registry/index.json").unwrap();
        assert_eq!(url.scheme(), "file");
        assert_eq!(url.to_string(), "file:///tmp/shuck%20registry/index.json");
        assert_eq!(
            url.to_file_path().unwrap(),
            PathBuf::from("/tmp/shuck registry/index.json")
        );
    }

    #[test]
    fn converts_directory_paths_to_urls() {
        let url = Url::from_directory_path("/tmp/shuck registry").unwrap();
        assert_eq!(url.to_string(), "file:///tmp/shuck%20registry/");
        assert_eq!(
            url.join("index.json").unwrap().to_string(),
            "file:///tmp/shuck%20registry/index.json"
        );
    }

    #[test]
    fn parses_file_urls() {
        let url = Url::parse("file:///etc/passwd").unwrap();
        assert_eq!(url.scheme(), "file");
        assert_eq!(url.path(), "/etc/passwd");
        assert_eq!(url.to_file_path().unwrap(), PathBuf::from("/etc/passwd"));
    }

    #[test]
    fn rejects_relative_file_paths() {
        assert!(Url::from_file_path("relative/path").is_err());
    }

    #[test]
    fn rejects_remote_hosts_for_file_urls() {
        let url = Url::parse("file://example.com/etc/passwd").unwrap();
        assert!(url.to_file_path().is_err());
    }

    #[test]
    fn rejects_malformed_urls() {
        for input in ["not a url", "/absolute/path", "https:/missing-authority"] {
            assert!(Url::parse(input).is_err(), "{input}");
        }
    }
}
