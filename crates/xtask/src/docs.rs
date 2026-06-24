//! Generate scoped Markdown API docs for a crate from its rustdoc comments.
//!
//! The crates carry thorough `///` and `//!` documentation; this turns that into committed,
//! browsable Markdown - one file per module plus an index - so the API is readable without
//! building rustdoc HTML. It parses the source with `syn` (no nightly needed), so it runs on
//! the scoop toolchain. `cargo xtask docs` regenerates the files; `cargo xtask docs --check`
//! re-generates in memory and fails if the committed docs are stale, the way the i18n bundles
//! are guarded.
//!
//! Scoped to `pamoja-dashboard` for now; the crate to document is a constant below.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use quote::ToTokens;
use syn::{Fields, ImplItem, Item, TraitItem, Visibility};

/// The crate whose API is documented, relative to the repo root, and where the docs go.
const CRATE_SRC: &str = "crates/pamoja-dashboard/src";
const DOCS_DIR: &str = "crates/pamoja-dashboard/docs/api";

/// Run the `docs` task: generate the API Markdown, or `--check` to verify it is in sync.
///
/// # Arguments
///
/// * `args` - `--check` verifies without writing; otherwise the docs are regenerated.
///
/// # Returns
///
/// Success when the docs were written, or when the check found them in sync.
pub fn run(args: &[String]) -> ExitCode {
    let check = args.iter().any(|arg| arg == "--check");
    match render_all() {
        Ok(files) => {
            if check {
                verify(&files)
            } else {
                write(&files)
            }
        }
        Err(message) => {
            eprintln!("xtask docs: {message}");
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

// Renders every output file as (relative path under DOCS_DIR, contents).
fn render_all() -> Result<Vec<(String, String)>, String> {
    let src = repo_root().join(CRATE_SRC);
    let mut modules: Vec<String> = fs::read_dir(&src)
        .map_err(|e| format!("reading {}: {e}", src.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_owned))
        .filter(|name| name != "lib")
        .collect();
    modules.sort();

    let mut files = Vec::new();
    for module in &modules {
        let source = fs::read_to_string(src.join(format!("{module}.rs")))
            .map_err(|e| format!("reading {module}.rs: {e}"))?;
        files.push((format!("{module}.md"), render_module(module, &source)?));
    }

    // The index: the crate overview from lib.rs, plus links to each module.
    let lib = fs::read_to_string(src.join("lib.rs")).map_err(|e| format!("reading lib.rs: {e}"))?;
    let lib_file = syn::parse_file(&lib).map_err(|e| format!("parsing lib.rs: {e}"))?;
    let mut index = String::from("# pamoja-dashboard API\n\n");
    index.push_str(
        "Generated from the crate's rustdoc by `cargo xtask docs` - do not edit by hand.\n\n",
    );
    let overview = doc_of(&lib_file.attrs);
    if !overview.is_empty() {
        index.push_str(&overview);
        index.push_str("\n\n");
    }
    index.push_str("## Modules\n\n");
    for module in &modules {
        index.push_str(&format!("- [{module}]({module}.md)\n"));
    }
    files.push(("README.md".to_owned(), index));
    Ok(files)
}

fn render_module(module: &str, source: &str) -> Result<String, String> {
    let file = syn::parse_file(source).map_err(|e| format!("parsing {module}.rs: {e}"))?;
    let mut out = format!("# {module}\n\n");
    out.push_str("Generated from rustdoc by `cargo xtask docs` - do not edit by hand.\n\n");
    let module_doc = doc_of(&file.attrs);
    if !module_doc.is_empty() {
        out.push_str(&module_doc);
        out.push_str("\n\n");
    }

    for item in &file.items {
        match item {
            Item::Struct(item) if is_public(&item.vis) => {
                section(
                    &mut out,
                    "struct",
                    &item.ident.to_string(),
                    &doc_of(&item.attrs),
                );
                fields(&mut out, &item.fields);
            }
            Item::Enum(item) if is_public(&item.vis) => {
                section(
                    &mut out,
                    "enum",
                    &item.ident.to_string(),
                    &doc_of(&item.attrs),
                );
                for variant in &item.variants {
                    let doc = doc_of(&variant.attrs);
                    out.push_str(&format!("- `{}`", variant.ident));
                    if !doc.is_empty() {
                        out.push_str(&format!(" - {}", doc.replace('\n', " ")));
                    }
                    out.push('\n');
                }
                out.push('\n');
            }
            Item::Trait(item) if is_public(&item.vis) => {
                section(
                    &mut out,
                    "trait",
                    &item.ident.to_string(),
                    &doc_of(&item.attrs),
                );
                for trait_item in &item.items {
                    if let TraitItem::Fn(method) = trait_item {
                        member(
                            &mut out,
                            &tidy(method.sig.to_token_stream().to_string()),
                            &doc_of(&method.attrs),
                        );
                    }
                }
            }
            Item::Fn(item) if is_public(&item.vis) => {
                section(
                    &mut out,
                    "fn",
                    &item.sig.ident.to_string(),
                    &doc_of(&item.attrs),
                );
                code(&mut out, &tidy(item.sig.to_token_stream().to_string()));
            }
            Item::Const(item) if is_public(&item.vis) => {
                section(
                    &mut out,
                    "const",
                    &item.ident.to_string(),
                    &doc_of(&item.attrs),
                );
                code(
                    &mut out,
                    &tidy(format!(
                        "const {}: {}",
                        item.ident,
                        item.ty.to_token_stream()
                    )),
                );
            }
            Item::Type(item) if is_public(&item.vis) => {
                section(
                    &mut out,
                    "type",
                    &item.ident.to_string(),
                    &doc_of(&item.attrs),
                );
            }
            // Inherent impls contribute their public methods under the type.
            Item::Impl(item) if item.trait_.is_none() => {
                let ty = tidy(item.self_ty.to_token_stream().to_string());
                for impl_item in &item.items {
                    if let ImplItem::Fn(method) = impl_item {
                        if is_public(&method.vis) {
                            out.push_str(&format!("### `{ty}::{}`\n\n", method.sig.ident));
                            let doc = doc_of(&method.attrs);
                            if !doc.is_empty() {
                                out.push_str(&doc);
                                out.push_str("\n\n");
                            }
                            code(&mut out, &tidy(method.sig.to_token_stream().to_string()));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(out)
}

fn section(out: &mut String, kind: &str, name: &str, doc: &str) {
    out.push_str(&format!("## {kind} `{name}`\n\n"));
    if !doc.is_empty() {
        out.push_str(doc);
        out.push_str("\n\n");
    }
}

fn member(out: &mut String, sig: &str, doc: &str) {
    out.push_str(&format!("### `{sig}`\n\n"));
    if !doc.is_empty() {
        out.push_str(doc);
        out.push_str("\n\n");
    }
}

fn code(out: &mut String, text: &str) {
    out.push_str(&format!("```rust\n{text}\n```\n\n"));
}

fn fields(out: &mut String, fields: &Fields) {
    if let Fields::Named(named) = fields {
        let public: Vec<_> = named.named.iter().filter(|f| is_public(&f.vis)).collect();
        if public.is_empty() {
            return;
        }
        out.push_str("Fields:\n\n");
        for field in public {
            let name = field
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            let ty = tidy(field.ty.to_token_stream().to_string());
            let doc = doc_of(&field.attrs);
            out.push_str(&format!("- `{name}: {ty}`"));
            if !doc.is_empty() {
                out.push_str(&format!(" - {}", doc.replace('\n', " ")));
            }
            out.push('\n');
        }
        out.push('\n');
    }
}

fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

// The rustdoc section headers that would otherwise render as top-level Markdown headings.
const SECTIONS: &[&str] = &[
    "# Arguments",
    "# Returns",
    "# Errors",
    "# Examples",
    "# Panics",
    "# Safety",
];

// Joins an item's `///`/`//!` doc lines (dropping the single leading space rustdoc adds) and
// renders them as Markdown: rustdoc section headers become bold labels rather than H1s, and
// hidden doctest lines (`#` inside a code fence) are dropped, with `##` unescaped to `#`.
fn doc_of(attrs: &[syn::Attribute]) -> String {
    let mut raw = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(text),
                    ..
                }) = &nv.value
                {
                    let line = text.value();
                    raw.push(line.strip_prefix(' ').unwrap_or(&line).to_owned());
                }
            }
        }
    }

    let mut out = Vec::new();
    let mut in_fence = false;
    for line in raw {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            out.push(line);
        } else if in_fence {
            if trimmed == "#" || trimmed.starts_with("# ") {
                continue; // a hidden doctest line
            }
            match trimmed.strip_prefix("##") {
                Some(rest) => out.push(format!("#{rest}")),
                None => out.push(line),
            }
        } else if SECTIONS.contains(&line.as_str()) {
            out.push(format!("**{}**", &line[2..]));
        } else {
            out.push(line);
        }
    }
    out.join("\n").trim_end().to_owned()
}

// Tidies a token-stream-rendered signature so it reads like source, not spaced tokens.
fn tidy(sig: String) -> String {
    sig.replace(" ::", "::")
        .replace(":: ", "::")
        .replace(" :", ":")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace(" ,", ",")
        .replace(" (", "(")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace("& ", "&")
        .replace("  ", " ")
}

fn write(files: &[(String, String)]) -> ExitCode {
    let dir = repo_root().join(DOCS_DIR);
    if let Err(err) = fs::create_dir_all(&dir) {
        eprintln!("xtask docs: creating {}: {err}", dir.display());
        return ExitCode::FAILURE;
    }
    for (name, body) in files {
        let path = dir.join(name);
        if let Err(err) = fs::write(&path, body) {
            eprintln!("xtask docs: writing {}: {err}", path.display());
            return ExitCode::FAILURE;
        }
        println!("wrote {}", path.display());
    }
    ExitCode::SUCCESS
}

fn verify(files: &[(String, String)]) -> ExitCode {
    let dir = repo_root().join(DOCS_DIR);
    let mut stale = Vec::new();
    for (name, body) in files {
        let path = dir.join(name);
        match fs::read_to_string(&path) {
            Ok(on_disk) if &on_disk == body => {}
            _ => stale.push(name.clone()),
        }
    }
    if stale.is_empty() {
        println!("docs: API Markdown is in sync");
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "xtask docs: stale or missing: {}\n  run `cargo xtask docs` and commit the result",
            stale.join(", ")
        );
        ExitCode::FAILURE
    }
}
