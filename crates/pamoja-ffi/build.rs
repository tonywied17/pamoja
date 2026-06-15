//! Regenerates the committed C header from the crate's `extern "C"` surface.
//!
//! `cbindgen` parses this crate and writes `include/pamoja.h`. The header is
//! checked into the tree and drift-checked in CI so it can never fall behind the
//! Rust source.
//!
//! The header is only refreshed when this crate is the primary package of the
//! build (a direct `cargo build`/`test`/`publish` of `pamoja-ffi`), never when it
//! is pulled in as a dependency, and the write is best-effort so a read-only
//! source tree (for example on docs.rs) cannot fail the build. CI is the gate that
//! enforces a fresh, committed header.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/mqtt.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    // Skip entirely when built as a dependency; only the direct build owns the
    // committed header.
    if env::var_os("CARGO_PRIMARY_PACKAGE").is_none() {
        return;
    }

    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config =
        cbindgen::Config::from_file(crate_dir.join("cbindgen.toml")).expect("read cbindgen.toml");

    let bindings = match cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
    {
        Ok(bindings) => bindings,
        Err(error) => {
            println!("cargo:warning=cbindgen could not generate the header: {error}");
            return;
        }
    };

    let mut rendered = Vec::new();
    bindings.write(&mut rendered);

    // Write only when the contents change, and treat any IO failure (such as a
    // read-only source tree) as a warning rather than a hard error.
    let header_path = crate_dir.join("include").join("pamoja.h");
    let unchanged = fs::read(&header_path)
        .map(|existing| existing == rendered)
        .unwrap_or(false);
    if unchanged {
        return;
    }
    if let Err(error) = fs::create_dir_all(header_path.parent().expect("include directory"))
        .and_then(|()| fs::write(&header_path, &rendered))
    {
        println!(
            "cargo:warning=could not write {}: {error}",
            header_path.display()
        );
    }
}
