//! Workspace task runner for pamoja.
//!
//! Run with `cargo xtask <task>`. The tasks are placeholders that document the
//! intended build and release automation; their implementations are filled in as
//! the bindings and packaging come online.

use std::process::ExitCode;

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
    ("release", "publish to crates.io, PyPI, npm, and NuGet"),
];

fn main() -> ExitCode {
    let Some(task) = std::env::args().nth(1) else {
        help();
        return ExitCode::SUCCESS;
    };

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

fn help() {
    println!("pamoja xtask");
    println!("usage: cargo xtask <task>\n");
    println!("tasks:");
    for (name, description) in TASKS {
        println!("  {name:<10} {description}");
    }
}
