//! The Zenoh key-expression language: validity, canonical form, and matching.
//!
//! A key expression is a `/`-joined list of non-empty chunks. A chunk is either a literal, the
//! single-chunk wildcard `*` (one non-empty chunk), the multi-chunk wildcard `**` (zero or more
//! chunks), or a literal carrying the sub-chunk wildcard `$*` (any run of characters, including
//! none, within one chunk). A concrete key carries no wildcards. Leading, trailing, and doubled
//! `/` are forbidden, as are the bare characters `*`, `$`, `?`, and `#` outside the wildcard forms.
//!
//! The rules follow the Zenoh key-expression specification, including its canonical-form rules:
//! `**/**` collapses to `**`, `**/*` reorders to `*/**`, `$*$*` collapses to `$*`, and a chunk that
//! is exactly `$*` becomes `*`.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Returns whether a string is a well-formed key expression.
///
/// # Arguments
///
/// * `ke` - the candidate key expression.
///
/// # Returns
///
/// `true` if `ke` is `/`-joined non-empty chunks with no leading, trailing, or doubled `/`, where
/// each chunk is `*`, `**`, or a literal in which `*` appears only as part of `$*` and `$` only
/// before `*`, with no `?` or `#`.
pub fn is_valid(ke: &str) -> bool {
    if ke.is_empty() || ke.starts_with('/') || ke.ends_with('/') {
        return false;
    }
    ke.split('/').all(chunk_valid)
}

/// Returns whether a key expression is valid and in canonical form.
///
/// # Arguments
///
/// * `ke` - the candidate key expression.
///
/// # Returns
///
/// `true` if `ke` equals its own [`canonize`] output, so two expressions selecting the same keys
/// compare equal as strings.
pub fn is_canon(ke: &str) -> bool {
    canonize(ke).as_deref() == Some(ke)
}

/// Returns the canonical form of a key expression, or `None` if it is invalid.
///
/// # Arguments
///
/// * `ke` - the key expression to canonicalize.
///
/// # Returns
///
/// `Some(canonical)` for a valid `ke`, applying the canonical-form rules (`**/**` to `**`, `**/*`
/// to `*/**`, `$*$*` to `$*`, and a `$*` chunk to `*`); `None` if `ke` is not a valid key
/// expression.
///
/// # Examples
///
/// ```
/// use pamoja_zenoh::keyexpr::canonize;
///
/// assert_eq!(canonize("robot/sensor/**/*").as_deref(), Some("robot/sensor/*/**"));
/// assert_eq!(canonize("a/**/**/b").as_deref(), Some("a/**/b"));
/// assert_eq!(canonize("a//b"), None); // a doubled slash is not a valid key expression
/// ```
pub fn canonize(ke: &str) -> Option<String> {
    if !is_valid(ke) {
        return None;
    }
    let canon_chunks: Vec<String> = ke.split('/').map(canon_chunk).collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < canon_chunks.len() {
        if is_wildcard(canon_chunks[i].as_str()) {
            let mut stars = 0;
            let mut has_multi = false;
            while i < canon_chunks.len() && is_wildcard(canon_chunks[i].as_str()) {
                if canon_chunks[i].as_str() == "*" {
                    stars += 1;
                } else {
                    has_multi = true;
                }
                i += 1;
            }
            out.extend((0..stars).map(|_| String::from("*")));
            if has_multi {
                out.push(String::from("**"));
            }
        } else {
            out.push(canon_chunks[i].clone());
            i += 1;
        }
    }
    Some(out.join("/"))
}

/// Returns whether a concrete key is selected by a pattern key expression.
///
/// # Arguments
///
/// * `pattern` - the key expression to test against; it may contain wildcards.
/// * `key` - the concrete key being routed; it must be valid and carry no wildcards.
///
/// # Returns
///
/// `true` if `key` is one of the keys `pattern` selects. Returns `false` if `pattern` is not a
/// valid key expression, or if `key` is not a valid concrete key.
///
/// # Examples
///
/// ```
/// use pamoja_zenoh::keyexpr::matches;
///
/// assert!(matches("room275/*/temperature", "room275/device1/temperature"));
/// assert!(!matches("room275/*/temperature", "room275/temperature")); // `*` needs one chunk
/// assert!(matches("organizationA/**/temperature", "organizationA/temperature")); // `**` allows none
/// assert!(matches("thermometer$*/temperature", "thermometer1/temperature"));
/// ```
pub fn matches(pattern: &str, key: &str) -> bool {
    if !is_valid(pattern) || !is_valid(key) || key.contains('*') {
        return false;
    }
    let pattern_chunks: Vec<&str> = pattern.split('/').collect();
    let key_chunks: Vec<&str> = key.split('/').collect();
    match_chunks(&pattern_chunks, &key_chunks)
}

fn is_wildcard(chunk: &str) -> bool {
    chunk == "*" || chunk == "**"
}

fn chunk_valid(chunk: &str) -> bool {
    if chunk.is_empty() {
        return false;
    }
    if is_wildcard(chunk) {
        return true;
    }
    let bytes = chunk.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'?' | b'#' | b'/' => return false,
            b'$' => {
                if i + 1 >= bytes.len() || bytes[i + 1] != b'*' {
                    return false;
                }
                i += 2;
            }
            b'*' => return false, // a `*` inside a chunk is legal only as part of `$*`
            _ => i += 1,
        }
    }
    true
}

fn canon_chunk(chunk: &str) -> String {
    if is_wildcard(chunk) {
        return chunk.to_string();
    }
    let mut s = chunk.to_string();
    while s.contains("$*$*") {
        s = s.replace("$*$*", "$*");
    }
    if s == "$*" {
        return String::from("*");
    }
    s
}

fn match_chunks(pattern: &[&str], key: &[&str]) -> bool {
    let Some((&head, rest)) = pattern.split_first() else {
        return key.is_empty();
    };
    if head == "**" {
        // `**` consumes zero or more key chunks; try every split.
        (0..=key.len()).any(|skip| match_chunks(rest, &key[skip..]))
    } else {
        match key.split_first() {
            Some((&first_key, rest_key)) => {
                chunk_matches(head, first_key) && match_chunks(rest, rest_key)
            }
            None => false,
        }
    }
}

fn chunk_matches(pattern: &str, literal: &str) -> bool {
    if pattern == "*" {
        return true; // any single non-empty chunk; the literal is non-empty by validity
    }
    glob_match(pattern, literal)
}

// Matches a single chunk pattern (literals plus `$*` sub-chunk wildcards) against a literal chunk.
fn glob_match(pattern: &str, s: &str) -> bool {
    if !pattern.contains("$*") {
        return pattern == s;
    }
    let parts: Vec<&str> = pattern.split("$*").collect();
    let first = parts[0];
    if !s.starts_with(first) {
        return false;
    }
    let mut idx = first.len();
    for part in &parts[1..parts.len() - 1] {
        if part.is_empty() {
            continue;
        }
        match s[idx..].find(part) {
            Some(pos) => idx += pos + part.len(),
            None => return false,
        }
    }
    let last = parts[parts.len() - 1];
    if last.is_empty() {
        return true;
    }
    s.len() >= idx + last.len() && s[idx..].ends_with(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validity_follows_the_chunk_rules() {
        assert!(is_valid("a/b/c"));
        assert!(is_valid("a/*/c"));
        assert!(is_valid("a/**/c"));
        assert!(is_valid("thermometer$*/temperature"));
        assert!(!is_valid("")); // empty
        assert!(!is_valid("/a")); // leading slash
        assert!(!is_valid("a/")); // trailing slash
        assert!(!is_valid("a//b")); // doubled slash
        assert!(!is_valid("a/b*")); // bare `*` inside a chunk
        assert!(!is_valid("a/$x")); // `$` not before `*`
        assert!(!is_valid("a/**b")); // `**` only as a whole chunk
        assert!(!is_valid("a/b?")); // `?` is reserved
    }

    #[test]
    fn matching_against_concrete_keys() {
        // The single-chunk wildcard needs exactly one chunk.
        assert!(matches(
            "room275/*/temperature",
            "room275/device1/temperature"
        ));
        assert!(!matches("room275/*/temperature", "room275/temperature"));
        assert!(!matches("room275/*/temperature", "room275/a/b/temperature"));

        // The multi-chunk wildcard spans zero or more chunks.
        assert!(matches(
            "organizationA/**/temperature",
            "organizationA/temperature"
        ));
        assert!(matches(
            "organizationA/**/temperature",
            "organizationA/b8/r275/temperature"
        ));

        // A leading `**` selects everything below a root.
        assert!(matches("**", "anything/at/all"));
        assert!(matches("demo/**", "demo/a/b/c"));
    }

    #[test]
    fn sub_chunk_wildcard_matches_within_a_chunk() {
        assert!(matches(
            "thermometer$*/temperature",
            "thermometer1/temperature"
        ));
        assert!(matches(
            "thermometer$*/temperature",
            "thermometerA/temperature"
        ));
        assert!(matches(
            "thermometer$*/temperature",
            "thermometer/temperature"
        )); // `$*` may be empty
        assert!(!matches(
            "thermometer$*/temperature",
            "xthermometer1/temperature"
        ));
        assert!(matches("a$*b$*c", "aXXbYYc"));
        assert!(!matches("a$*b$*c", "aXXc")); // the middle `b` is missing
    }

    #[test]
    fn a_pattern_does_not_match_a_key_with_wildcards() {
        // The right-hand side must be a concrete key.
        assert!(!matches("a/*", "a/*"));
        assert!(!matches("a/b", "a/*"));
    }

    #[test]
    fn canonical_form_examples() {
        // The published reordering example.
        assert_eq!(
            canonize("robot/sensor/**/*").as_deref(),
            Some("robot/sensor/*/**")
        );
        // Consecutive `**` collapse.
        assert_eq!(canonize("a/**/**/b").as_deref(), Some("a/**/b"));
        // Consecutive `$*` collapse, and a lone `$*` chunk becomes `*`.
        assert_eq!(canonize("a/x$*$*y/$*").as_deref(), Some("a/x$*y/*"));
        // A mixed wildcard run sorts the single wildcards ahead of the multi.
        assert_eq!(canonize("**/*/*").as_deref(), Some("*/*/**"));

        assert!(is_canon("robot/sensor/*/**"));
        assert!(!is_canon("robot/sensor/**/*"));
        assert!(!is_canon("a//b")); // invalid is never canonical
    }
}
