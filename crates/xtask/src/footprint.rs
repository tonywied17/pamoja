//! Dashboard footprint check: enforce the gzipped transfer budget for the page-load bundle.
//!
//! "Best-looking on cheap hardware" means fast and tiny, so `.docs/LOCAL_DASHBOARDS.md`
//! sets a hard transfer budget. This sums the gzipped size of what a browser actually
//! fetches on first load - the shell, the styles, every script, and one locale - and fails
//! if it grows past the budget, so the size is visible on every run and a regression is
//! caught in CI rather than in the field.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use flate2::write::GzEncoder;
use flate2::Compression;

/// The largest the page-load bundle may be once gzipped, including one locale. The plan's
/// full-tier target; current bundles sit well under it.
const BUDGET: usize = 150 * 1024;

/// Run the `dashboard footprint` task: report and enforce the gzipped bundle budget.
///
/// # Arguments
///
/// * `args` - ignored; the check takes no options.
///
/// # Returns
///
/// Success when the bundle is within budget, otherwise a failure with the overage.
pub fn run(args: &[String]) -> ExitCode {
    let _ = args;
    match check() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask dashboard footprint: {message}");
            ExitCode::FAILURE
        }
    }
}

fn web_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root is two levels above the xtask crate")
        .join("crates/pamoja-dashboard/web")
}

// The files a browser fetches on first load: the shell, styles, every script, and one
// locale (the budget is "including one locale"); other locales load only when chosen.
fn page_load_files(web: &Path) -> Result<Vec<PathBuf>, String> {
    let mut all = Vec::new();
    collect(web, &mut all).map_err(|e| format!("walking {}: {e}", web.display()))?;
    let mut chosen: Vec<PathBuf> = all
        .into_iter()
        .filter(|p| {
            matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("html") | Some("css") | Some("js")
            )
        })
        .collect();
    chosen.push(web.join("app/i18n/en.json"));
    chosen.sort();
    Ok(chosen)
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn check() -> Result<(), String> {
    let web = web_dir();
    let mut rows = Vec::new();
    let mut total = 0usize;
    for path in page_load_files(&web)? {
        let bytes = fs::read(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let size = gzipped_len(&bytes);
        total += size;
        let rel = path
            .strip_prefix(&web)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        rows.push((size, rel));
    }
    rows.sort_by_key(|row| std::cmp::Reverse(row.0));
    for (size, rel) in &rows {
        println!("  {size:>6}  {rel}");
    }
    println!("dashboard footprint: {total} bytes gzipped, budget {BUDGET}");
    if total > BUDGET {
        Err(format!(
            "page-load bundle is {total} bytes gzipped, over the {BUDGET} budget by {}",
            total - BUDGET
        ))
    } else {
        Ok(())
    }
}

fn gzipped_len(bytes: &[u8]) -> usize {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(bytes).expect("gzip write");
    encoder.finish().expect("gzip finish").len()
}
