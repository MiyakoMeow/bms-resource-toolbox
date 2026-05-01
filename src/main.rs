//! BMS Resource Toolbox - Entry point.
//!
//! Launches TUI by default, or dispatches CLI subcommands.

use std::io::IsTerminal as _;

use clap::Parser;

mod bms;
mod cli;
mod error;
mod fs;
mod media;
mod options;
mod scripts;
mod tui;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();

    if let Some(command) = &cli.command {
        cli::dispatch(command);
    } else if cli.tui || std::io::stdin().is_terminal() && tui::run_tui().is_ok() {
        // --tui forced, or default TUI succeeded
    } else {
        eprintln!("TUI unavailable and no subcommand given. Use --help for usage.");
        std::process::exit(1);
    }
}
