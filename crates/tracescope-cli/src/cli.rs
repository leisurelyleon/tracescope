//! Command-line argument definitions.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// A tracing and performance-analysis toolkit.
#[derive(Debug, Parser)]
#[command(name = "tracescope", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Analyze a JSON trace file and print a performance report.
    Analyze {
        /// Path to a JSON trace file.
        path: PathBuf,
    },
    /// Run a built-in instrumented demo workload and print its analysis.
    Demo,
}
