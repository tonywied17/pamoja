//! Dashboard localization tooling: generate the browser locale bundles from the
//! canonical Fluent store, and guard them in CI.
//!
//! `cargo xtask dashboard i18n` parses `crates/pamoja-i18n/i18n/*.ftl` and writes the
//! drop-in JS bundles the dashboard imports. `--check` regenerates in memory and runs the
//! core guards (generated-in-sync, key parity, variable parity, footprint) without
//! touching the tree, so CI fails on drift instead of shipping it.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use flate2::write::GzEncoder;
use flate2::Compression;
use fluent_syntax::ast;
use fluent_syntax::parser;

/// A shipped locale and the display metadata the bundle carries. Mirrors the table in
/// `pamoja-i18n`; both read the same `.ftl` store.
struct Locale {
    tag: &'static str,
    dir: &'static str,
    numbering: &'static str,
}

/// The seed locales, English first as the source of canonical key order.
const LOCALES: &[Locale] = &[
    Locale {
        tag: "en",
        dir: "ltr",
        numbering: "latn",
    },
    Locale {
        tag: "sw",
        dir: "ltr",
        numbering: "latn",
    },
    Locale {
        tag: "ar",
        dir: "rtl",
        numbering: "arab",
    },
    Locale {
        tag: "fr",
        dir: "ltr",
        numbering: "latn",
    },
    Locale {
        tag: "pt",
        dir: "ltr",
        numbering: "latn",
    },
    Locale {
        tag: "hi",
        dir: "ltr",
        numbering: "deva",
    },
];

/// The largest a single emitted bundle may be once gzipped. Current bundles sit far
/// under this; the budget catches a regression, not normal growth.
const FOOTPRINT_BUDGET: usize = 6 * 1024;

/// Run the `dashboard i18n` task: emit by default, or `--check` to verify only.
///
/// # Arguments
///
/// * `args` - the arguments after `i18n`; `--check` switches to verification.
///
/// # Returns
///
/// Success when emission wrote the bundles, or when every check passed.
pub fn run(args: &[String]) -> ExitCode {
    let check = args.iter().any(|arg| arg == "--check");
    match if check { check_all() } else { emit_all() } {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask dashboard i18n: {message}");
            ExitCode::FAILURE
        }
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root is two levels above the xtask crate")
        .to_path_buf()
}

fn ftl_dir() -> PathBuf {
    repo_root().join("crates/pamoja-i18n/i18n")
}

fn js_dir() -> PathBuf {
    repo_root().join("crates/pamoja-dashboard/web/app/i18n")
}

fn parse_locale(tag: &str) -> Result<ast::Resource<String>, String> {
    let path = ftl_dir().join(format!("{tag}.ftl"));
    let source =
        fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    parser::parse(source).map_err(|(_, errors)| format!("parsing {tag}.ftl: {errors:?}"))
}

// The messages of a resource, by id, in source order.
fn messages(resource: &ast::Resource<String>) -> Vec<&ast::Message<String>> {
    resource
        .body
        .iter()
        .filter_map(|entry| match entry {
            ast::Entry::Message(message) => Some(message),
            _ => None,
        })
        .collect()
}

fn id_set(resource: &ast::Resource<String>) -> BTreeSet<String> {
    messages(resource)
        .iter()
        .map(|message| message.id.name.clone())
        .collect()
}

/// Generates every bundle and writes it to the dashboard's `i18n` directory.
fn emit_all() -> Result<(), String> {
    let out_dir = js_dir();
    fs::create_dir_all(&out_dir).map_err(|e| format!("creating {}: {e}", out_dir.display()))?;
    for (tag, js) in render_all()? {
        let path = out_dir.join(format!("{tag}.js"));
        fs::write(&path, js).map_err(|e| format!("writing {}: {e}", path.display()))?;
        println!("emitted {tag}.js");
    }
    Ok(())
}

/// Renders every bundle in memory, keyed by locale tag.
fn render_all() -> Result<Vec<(&'static str, String)>, String> {
    let english = parse_locale("en")?;
    let order: Vec<String> = messages(&english)
        .iter()
        .map(|message| message.id.name.clone())
        .collect();

    let mut out = Vec::new();
    for locale in LOCALES {
        let resource = parse_locale(locale.tag)?;
        let by_id: HashMap<String, &ast::Message<String>> = messages(&resource)
            .into_iter()
            .map(|message| (message.id.name.clone(), message))
            .collect();
        out.push((locale.tag, render_bundle(locale, &order, &by_id)?));
    }
    Ok(out)
}

fn render_bundle(
    locale: &Locale,
    order: &[String],
    by_id: &HashMap<String, &ast::Message<String>>,
) -> Result<String, String> {
    let mut out = String::new();
    out.push_str(&format!(
        "/*\n  {} bundle - GENERATED from crates/pamoja-i18n/i18n/{}.ftl by\n  `cargo xtask dashboard i18n`. Do not edit by hand; edit the .ftl and regenerate.\n*/\n",
        locale.tag, locale.tag
    ));
    out.push_str("export default\n{\n");
    out.push_str(&format!("  locale: '{}',\n", locale.tag));
    out.push_str(&format!("  dir: '{}',\n", locale.dir));
    out.push_str(&format!("  numberingSystem: '{}',\n", locale.numbering));
    out.push_str("  messages:\n  {\n");
    for id in order {
        let Some(message) = by_id.get(id) else {
            continue;
        };
        let pattern = message
            .value
            .as_ref()
            .ok_or_else(|| format!("{}: message {id} has no value", locale.tag))?;
        let key = id.replace("__", ".");
        let compiled = compile_pattern(pattern)?;
        match compiled {
            Compiled::Const(text) => {
                out.push_str(&format!("    {}: {},\n", js_key(&key), js_string(&text)))
            }
            Compiled::Func(expr) => {
                out.push_str(&format!("    {}: (a, h) => {expr},\n", js_key(&key)))
            }
        }
    }
    out.push_str("  },\n};\n");
    Ok(out)
}

// A compiled message: either a constant string or a JS arrow-body expression that takes
// the args object `a` and the runtime helpers `h`.
enum Compiled {
    Const(String),
    Func(String),
}

fn compile_pattern(pattern: &ast::Pattern<String>) -> Result<Compiled, String> {
    // A single placeable wrapping a select compiles to the select expression directly,
    // which is the common counted-message shape.
    if let [ast::PatternElement::Placeable {
        expression: ast::Expression::Select { selector, variants },
    }] = pattern.elements.as_slice()
    {
        return Ok(Compiled::Func(compile_select(selector, variants)?));
    }

    let mut is_const = true;
    let mut const_text = String::new();
    let mut template = String::new();
    for element in &pattern.elements {
        match element {
            ast::PatternElement::TextElement { value } => {
                const_text.push_str(value);
                template.push_str(&js_template_text(value));
            }
            ast::PatternElement::Placeable { expression } => match constant_of(expression) {
                Some(text) => {
                    const_text.push_str(&text);
                    template.push_str(&js_template_text(&text));
                }
                None => {
                    is_const = false;
                    template.push_str(&format!("${{{}}}", compile_dynamic(expression)?));
                }
            },
        }
    }
    if is_const {
        Ok(Compiled::Const(const_text))
    } else {
        Ok(Compiled::Func(format!("`{template}`")))
    }
}

// The constant string an expression resolves to, if it has no runtime inputs.
fn constant_of(expression: &ast::Expression<String>) -> Option<String> {
    match expression {
        ast::Expression::Inline(ast::InlineExpression::StringLiteral { value }) => {
            Some(value.clone())
        }
        ast::Expression::Inline(ast::InlineExpression::NumberLiteral { value }) => {
            Some(value.clone())
        }
        _ => None,
    }
}

fn compile_dynamic(expression: &ast::Expression<String>) -> Result<String, String> {
    match expression {
        ast::Expression::Select { selector, variants } => compile_select(selector, variants),
        ast::Expression::Inline(inline) => compile_inline(inline),
    }
}

fn compile_inline(inline: &ast::InlineExpression<String>) -> Result<String, String> {
    match inline {
        ast::InlineExpression::VariableReference { id } => Ok(format!("a.{}", id.name)),
        ast::InlineExpression::FunctionReference { id, arguments } if id.name == "NUMBER" => {
            let first = arguments
                .positional
                .first()
                .ok_or("NUMBER() needs one argument")?;
            Ok(format!("h.num({})", compile_inline(first)?))
        }
        ast::InlineExpression::StringLiteral { value } => Ok(js_string(value)),
        ast::InlineExpression::NumberLiteral { value } => Ok(value.clone()),
        other => Err(format!("unsupported inline expression: {other:?}")),
    }
}

fn compile_select(
    selector: &ast::InlineExpression<String>,
    variants: &[ast::Variant<String>],
) -> Result<String, String> {
    let var = selector_var(selector)?;
    let mut entries = Vec::new();
    let mut default_key = String::from("other");
    for variant in variants {
        let key = match &variant.key {
            ast::VariantKey::Identifier { name } => name.clone(),
            ast::VariantKey::NumberLiteral { value } => value.clone(),
        };
        if variant.default {
            default_key = key.clone();
        }
        let value = match compile_pattern(&variant.value)? {
            Compiled::Const(text) => js_string(&text),
            Compiled::Func(expr) => expr,
        };
        entries.push(format!("{}: {value}", js_key(&key)));
    }
    Ok(format!(
        "h.sel(a.{var}, {{ {} }}, {})",
        entries.join(", "),
        js_string(&default_key),
    ))
}

fn selector_var(selector: &ast::InlineExpression<String>) -> Result<String, String> {
    match selector {
        ast::InlineExpression::VariableReference { id } => Ok(id.name.clone()),
        ast::InlineExpression::FunctionReference { id, arguments } if id.name == "NUMBER" => {
            match arguments.positional.first() {
                Some(ast::InlineExpression::VariableReference { id }) => Ok(id.name.clone()),
                _ => Err("NUMBER() selector needs a variable argument".into()),
            }
        }
        other => Err(format!("unsupported selector: {other:?}")),
    }
}

// Collects the variable names a message references, for variable-parity checking.
fn pattern_vars(pattern: &ast::Pattern<String>, out: &mut BTreeSet<String>) {
    for element in &pattern.elements {
        if let ast::PatternElement::Placeable { expression } = element {
            expression_vars(expression, out);
        }
    }
}

fn expression_vars(expression: &ast::Expression<String>, out: &mut BTreeSet<String>) {
    match expression {
        ast::Expression::Inline(inline) => inline_vars(inline, out),
        ast::Expression::Select { selector, variants } => {
            inline_vars(selector, out);
            for variant in variants {
                pattern_vars(&variant.value, out);
            }
        }
    }
}

fn inline_vars(inline: &ast::InlineExpression<String>, out: &mut BTreeSet<String>) {
    match inline {
        ast::InlineExpression::VariableReference { id } => {
            out.insert(id.name.clone());
        }
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            for positional in &arguments.positional {
                inline_vars(positional, out);
            }
        }
        _ => {}
    }
}

fn js_key(key: &str) -> String {
    js_string(key)
}

fn js_string(text: &str) -> String {
    let mut out = String::from("'");
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(c),
        }
    }
    out.push('\'');
    out
}

fn js_template_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${")
}

/// Runs the core localization guards against the canonical store and emitted bundles.
fn check_all() -> Result<(), String> {
    let mut failures = Vec::new();

    // Key parity and variable parity, comparing every locale to English.
    let english = parse_locale("en")?;
    let english_ids = id_set(&english);
    let english_vars = var_map(&english);
    for locale in LOCALES.iter().filter(|l| l.tag != "en") {
        let resource = parse_locale(locale.tag)?;
        let ids = id_set(&resource);
        for missing in english_ids.difference(&ids) {
            failures.push(format!("{}: missing key {missing}", locale.tag));
        }
        for extra in ids.difference(&english_ids) {
            failures.push(format!("{}: unexpected key {extra}", locale.tag));
        }
        for (id, vars) in var_map(&resource) {
            if let Some(expected) = english_vars.get(&id) {
                if &vars != expected {
                    failures.push(format!(
                        "{}: {id} uses variables {vars:?}, English uses {expected:?}",
                        locale.tag
                    ));
                }
            }
        }
    }

    // Generated-in-sync and footprint, over the freshly rendered bundles.
    for (tag, js) in render_all()? {
        let path = js_dir().join(format!("{tag}.js"));
        match fs::read_to_string(&path) {
            Ok(on_disk) if on_disk == js => {}
            Ok(_) => failures.push(format!(
                "{tag}.js is stale; run `cargo xtask dashboard i18n` and commit the result"
            )),
            Err(e) => failures.push(format!("reading {}: {e}", path.display())),
        }
        let size = gzipped_len(&js);
        if size > FOOTPRINT_BUDGET {
            failures.push(format!(
                "{tag}.js is {size} bytes gzipped, over the {FOOTPRINT_BUDGET} budget"
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

fn var_map(resource: &ast::Resource<String>) -> HashMap<String, BTreeSet<String>> {
    messages(resource)
        .iter()
        .filter_map(|message| {
            message.value.as_ref().map(|pattern| {
                let mut vars = BTreeSet::new();
                pattern_vars(pattern, &mut vars);
                (message.id.name.clone(), vars)
            })
        })
        .collect()
}

fn gzipped_len(text: &str) -> usize {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(text.as_bytes()).expect("gzip write");
    encoder.finish().expect("gzip finish").len()
}
