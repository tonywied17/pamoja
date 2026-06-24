//! Dashboard localization check: validate the per-locale JSON bundles.
//!
//! The dashboard's translations live as one self-describing JSON file per locale under
//! `crates/pamoja-dashboard/web/app/i18n/`, fetched directly by the browser. There is no
//! generation step. This guards them in CI so they cannot drift: every locale carries
//! English's keys and the same `{placeholders}` per message, declares its metadata, and
//! stays under its gzipped footprint budget. `cargo xtask dashboard i18n` (with or without
//! `--check`) runs the same checks.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::{Map, Value};

/// The shipped locales, English first as the reference key set.
const LOCALES: &[&str] = &["en", "sw", "ar", "fr", "pt", "hi"];

/// The largest a single bundle may be once gzipped. Current bundles sit far under this;
/// the budget catches a regression, not normal growth.
const FOOTPRINT_BUDGET: usize = 6 * 1024;

/// Run the `dashboard i18n` task: validate the locale bundles.
///
/// # Arguments
///
/// * `args` - ignored; `i18n` and `i18n --check` both validate, since nothing is generated.
///
/// # Returns
///
/// Success when every bundle passes, otherwise a failure with the problems listed.
pub fn run(args: &[String]) -> ExitCode {
    let _ = args;
    match check_all() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask dashboard i18n: {message}");
            ExitCode::FAILURE
        }
    }
}

fn i18n_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root is two levels above the xtask crate")
        .join("crates/pamoja-dashboard/web/app/i18n")
}

fn read(tag: &str) -> Result<(String, Value), String> {
    let path = i18n_dir().join(format!("{tag}.json"));
    let raw = fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let value = serde_json::from_str(&raw).map_err(|e| format!("parsing {tag}.json: {e}"))?;
    Ok((raw, value))
}

fn messages(bundle: &Value) -> Option<&Map<String, Value>> {
    bundle.get("messages").and_then(Value::as_object)
}

// Collects the `{name}` placeholders a message uses, across a string or a plural map.
fn placeholders(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::String(text) => extract(text, out),
        Value::Object(map) => {
            for variant in map.values() {
                if let Value::String(text) = variant {
                    extract(text, out);
                }
            }
        }
        _ => {}
    }
}

fn extract(text: &str, out: &mut BTreeSet<String>) {
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('}') else { break };
        let name = &rest[..close];
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            out.insert(name.to_owned());
        }
        rest = &rest[close + 1..];
    }
}

fn check_all() -> Result<(), String> {
    let mut failures = Vec::new();

    let (_, english) = read("en")?;
    let english_messages = messages(&english).ok_or("en.json: missing messages object")?;
    let english_keys: BTreeSet<&String> = english_messages.keys().collect();

    for &tag in LOCALES {
        let (raw, bundle) = read(tag)?;

        for field in ["locale", "dir", "numberingSystem", "messages"] {
            if bundle.get(field).is_none() {
                failures.push(format!("{tag}.json: missing \"{field}\""));
            }
        }

        let Some(locale_messages) = messages(&bundle) else {
            failures.push(format!("{tag}.json: messages is not an object"));
            continue;
        };

        if tag != "en" {
            let keys: BTreeSet<&String> = locale_messages.keys().collect();
            for missing in english_keys.difference(&keys) {
                failures.push(format!("{tag}: missing key {missing}"));
            }
            for extra in keys.difference(&english_keys) {
                failures.push(format!("{tag}: unexpected key {extra}"));
            }
            for (key, value) in locale_messages {
                if let Some(english_value) = english_messages.get(key) {
                    let mut here = BTreeSet::new();
                    placeholders(value, &mut here);
                    let mut there = BTreeSet::new();
                    placeholders(english_value, &mut there);
                    if here != there {
                        failures.push(format!(
                            "{tag}: {key} uses placeholders {here:?}, English uses {there:?}"
                        ));
                    }
                }
            }
        }

        let size = gzipped_len(&raw);
        if size > FOOTPRINT_BUDGET {
            failures.push(format!(
                "{tag}.json is {size} bytes gzipped, over the {FOOTPRINT_BUDGET} budget"
            ));
        }
    }

    if failures.is_empty() {
        println!("dashboard i18n: all checks passed");
        Ok(())
    } else {
        Err(failures.join("\n  "))
    }
}

fn gzipped_len(text: &str) -> usize {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(text.as_bytes()).expect("gzip write");
    encoder.finish().expect("gzip finish").len()
}
