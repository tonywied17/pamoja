//! Generates the static fleet snapshots the hosted showcase replays when no device answers
//! `GET /state`. For each mock scenario it advances the clock and writes a JSON array of
//! frames to `state.<scenario>.json`; the static dashboard cycles them on the live cadence
//! so it animates like the device feed. Used by the Pages build:
//! `cargo run -p pamoja-dashboard --example snapshot -- dist/dashboard`.

use std::path::PathBuf;
use std::process::ExitCode;

use pamoja_dashboard::{Mock, Scenario, StateSource};

const FRAMES: usize = 48;

fn main() -> ExitCode {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".to_owned()));
    if let Err(err) = std::fs::create_dir_all(&out) {
        eprintln!("snapshot: could not create {}: {err}", out.display());
        return ExitCode::FAILURE;
    }

    for scenario in Scenario::ALL {
        let mut mock = Mock::new(scenario);
        let mut frames = Vec::with_capacity(FRAMES);
        for _ in 0..FRAMES {
            match mock.snapshot().to_json() {
                Ok(json) => frames.push(json),
                Err(err) => {
                    eprintln!("snapshot: could not serialize {}: {err}", scenario.key());
                    return ExitCode::FAILURE;
                }
            }
        }
        let body = format!("[{}]", frames.join(","));
        let path = out.join(format!("state.{}.json", scenario.key()));
        if let Err(err) = std::fs::write(&path, body) {
            eprintln!("snapshot: could not write {}: {err}", path.display());
            return ExitCode::FAILURE;
        }
        println!("wrote {} ({FRAMES} frames)", path.display());
    }

    ExitCode::SUCCESS
}
