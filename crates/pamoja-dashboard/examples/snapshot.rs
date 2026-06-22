//! Generates the static fleet snapshots the hosted showcase loads when no device answers
//! `GET /state`. Writes `state.<scenario>.json` for every mock scenario into the output
//! directory (the first argument, default the current directory). Used by the Pages build:
//! `cargo run -p pamoja-dashboard --example snapshot -- dist/dashboard`.

use std::path::PathBuf;
use std::process::ExitCode;

use pamoja_dashboard::{Mock, Scenario, StateSource};

fn main() -> ExitCode {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".to_owned()));
    if let Err(err) = std::fs::create_dir_all(&out) {
        eprintln!("snapshot: could not create {}: {err}", out.display());
        return ExitCode::FAILURE;
    }

    for scenario in Scenario::ALL {
        let mut mock = Mock::new(scenario);
        let json = match mock.snapshot().to_json() {
            Ok(json) => json,
            Err(err) => {
                eprintln!("snapshot: could not serialize {}: {err}", scenario.key());
                return ExitCode::FAILURE;
            }
        };
        let path = out.join(format!("state.{}.json", scenario.key()));
        if let Err(err) = std::fs::write(&path, json) {
            eprintln!("snapshot: could not write {}: {err}", path.display());
            return ExitCode::FAILURE;
        }
        println!("wrote {}", path.display());
    }

    ExitCode::SUCCESS
}
