//! Workspace task runner for pamoja.
//!
//! Run with `cargo xtask <task>`. Most tasks are placeholders that document the
//! intended build automation; `release` is implemented and publishes the
//! workspace crates to crates.io in dependency order, riding out the registry's
//! new-crate rate limit so a first-time publish of the whole workspace completes
//! in a single run.

use std::process::{Command, ExitCode};
use std::thread::sleep;
use std::time::Duration;

/// The tasks xtask knows about, each paired with a one-line description.
const TASKS: &[(&str, &str)] = &[
    (
        "codegen",
        "regenerate every language binding from the Rust core",
    ),
    ("build-all", "build the core plus every language binding"),
    (
        "test-all",
        "run Rust tests plus the cross-language conformance suite",
    ),
    (
        "package",
        "produce wheels, Node prebuilds, and the NuGet package",
    ),
    (
        "release",
        "publish the workspace crates to crates.io in dependency order",
    ),
];

/// Publishable crates in dependency order: a crate never appears before one it
/// depends on, so each is resolvable on crates.io by the time a dependent is
/// published. `xtask` and `examples` are not published and are absent.
const RELEASE_ORDER: &[&str] = &[
    "pamoja-core",
    "pamoja-codec",
    "pamoja-mqtt",
    "pamoja-coap",
    "pamoja-ffi",
    "pamoja-sync",
    "pamoja-loopback",
    "pamoja-bus",
    "pamoja-ladder",
    "pamoja-kit",
    "pamoja-power",
    "pamoja-profile",
    "pamoja-sim",
    "pamoja-security",
    "pamoja-audit",
    "pamoja-telemetry",
    "pamoja-lora",
    "pamoja-modbus",
    "pamoja-mesh",
    "pamoja-lorawan",
    "pamoja-routing",
    "pamoja-can",
    "pamoja-serial",
];

/// Seconds to wait before retrying a crate that crates.io throttled. The
/// new-crate limit refills one crate every ten minutes, so the default waits a
/// little past that. Override with `PAMOJA_RELEASE_RETRY_SECS`.
const DEFAULT_RETRY_SECS: u64 = 660;

/// Attempts per crate before giving up, so a persistent failure cannot loop
/// forever.
const MAX_ATTEMPTS: u32 = 12;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(task) = args.next() else {
        help();
        return ExitCode::SUCCESS;
    };

    if task == "release" {
        return release(&args.collect::<Vec<_>>());
    }

    match TASKS.iter().find(|(name, _)| *name == task) {
        Some((name, description)) => {
            println!("xtask {name}: not implemented yet ({description}).");
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("unknown task: {task}\n");
            help();
            ExitCode::FAILURE
        }
    }
}

/// Publish every crate in `RELEASE_ORDER`. A version already on crates.io is
/// skipped, and a crate throttled by the new-crate rate limit is retried after a
/// wait, so the whole workspace publishes in one run even though new crates are
/// limited to one every ten minutes. Pass `--dry-run` to package and verify each
/// crate without uploading. Reads the token from `CARGO_REGISTRY_TOKEN`.
fn release(args: &[String]) -> ExitCode {
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let retry_secs = std::env::var("PAMOJA_RELEASE_RETRY_SECS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_RETRY_SECS);

    let count = RELEASE_ORDER.len();
    if dry_run {
        println!("xtask release: dry run, packaging {count} crates without uploading\n");
    } else {
        println!("xtask release: publishing {count} crates to crates.io\n");
    }

    for crate_name in RELEASE_ORDER {
        if !publish(crate_name, dry_run, retry_secs) {
            return ExitCode::FAILURE;
        }
    }

    println!("\nxtask release: all {count} crates are published");
    ExitCode::SUCCESS
}

/// Publish a single crate, retrying while crates.io reports its rate limit.
/// Returns `true` once the crate is published or already present, `false` on a
/// real failure.
fn publish(crate_name: &str, dry_run: bool, retry_secs: u64) -> bool {
    for attempt in 1..=MAX_ATTEMPTS {
        println!("==> {crate_name} (attempt {attempt})");

        let mut cmd = Command::new("cargo");
        cmd.arg("publish").arg("-p").arg(crate_name);
        if dry_run {
            cmd.arg("--dry-run").arg("--allow-dirty");
        }

        let output = match cmd.output() {
            Ok(output) => output,
            Err(err) => {
                eprintln!("could not run cargo for {crate_name}: {err}");
                return false;
            }
        };

        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        print!("{combined}");

        if output.status.success() {
            println!("published {crate_name}\n");
            return true;
        }

        let report = combined.to_lowercase();
        if report.contains("already uploaded") || report.contains("already exists") {
            println!("skipping {crate_name}: this version is already on crates.io\n");
            return true;
        }

        if report.contains("rate limit") || report.contains("too many") || report.contains("429") {
            println!("{crate_name} hit the crates.io rate limit; waiting {retry_secs}s before retry\n");
            sleep(Duration::from_secs(retry_secs));
            continue;
        }

        eprintln!("failed to publish {crate_name}");
        return false;
    }

    eprintln!("gave up on {crate_name} after {MAX_ATTEMPTS} attempts");
    false
}

fn help() {
    println!("pamoja xtask");
    println!("usage: cargo xtask <task>\n");
    println!("tasks:");
    for (name, description) in TASKS {
        println!("  {name:<10} {description}");
    }
}
