//! `tracescope` command-line entry point.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;

use tracescope_cli::cli::{Cli, Command};
use tracescope_collect::{Collector, ManualClock};
use tracescope_core::{analyze, Trace};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Analyze { path } => match run_analyze(&path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("tracescope: {message}");
                ExitCode::FAILURE
            }
        },
        Command::Demo => {
            run_demo();
            ExitCode::SUCCESS
        }
    }
}

fn run_analyze(path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("could not read {}: {err}", path.display()))?;
    let trace = Trace::from_json(&text).map_err(|err| err.to_string())?;
    trace.validate().map_err(|err| err.to_string())?;
    let report = analyze(&trace);
    println!("{}", report.format_text());
    Ok(())
}

/// Builds a deterministic instrumented workload (via a manual clock), analyzes
/// it, and prints the report — so the demo output is stable and inspectable.
fn run_demo() {
    let collector = Collector::new(ManualClock::new());
    {
        let _request = collector.span("handle_request"); // t=0
        collector.clock().advance(5);
        {
            let _db = collector.span("db_query"); // t=5
            collector.clock().advance(40);
        } // db_query ends t=45
        {
            let _render = collector.span("render"); // t=45
            collector.clock().advance(15);
            {
                let _serialize = collector.span("serialize"); // t=60
                collector.clock().advance(10);
            } // serialize ends t=70
        } // render ends t=70
        collector.clock().advance(5);
    } // handle_request ends t=75

    let trace = collector.finish();
    let report = analyze(&trace);
    println!("{}", report.format_text());
}
