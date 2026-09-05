//! # Error Handling Without Exiting
//!
//! This example shows library-style parsing with `xarp`:
//! - Parsing with [`Xarp::try_get_matches`] instead of the exiting `get_matches`.
//! - Branching on [`XarpError`] via `is_help`, `is_version`, and `is_parse`.
//! - Distinguishing a missing value from an invalid one with `try_get_one`.
//! - Mapping outcomes to conventional exit codes (`0` for help/version, `2`
//!   for usage errors).
//!
//! Try it with:
//! - `cargo run -q --example error_handling -- --help`
//! - `cargo run -q --example error_handling -- --port 8080 --mode fast`
//! - `cargo run -q --example error_handling -- --port abc` (typed-value error)
//! - `cargo run -q --example error_handling -- --mode wild` (possible-values error)

use std::process::ExitCode;
use xarp::{Arg, ArgAction, Xarp};

/// Builds the demo CLI definition shared by every parse below.
fn build_app() -> Xarp {
    Xarp::new("deploy")
        .version("0.1.2-dev")
        .about("Fake deployment tool used to demonstrate error handling")
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("TCP port to listen on")
                .default_value("8080"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_name("MODE")
                .help("Deployment mode")
                .possible_values(["fast", "safe"])
                .default_value("safe"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::SetTrue),
        )
}

/// Parses the process arguments and prints the resolved configuration.
///
/// Help and version payloads already contain rendered text, so they are
/// printed as-is. Everything else is a usage error.
fn main() -> ExitCode {
    let matches = match build_app().try_get_matches() {
        Ok(matches) => matches,
        Err(err) if err.is_help() || err.is_version() => {
            print!("{err}");
            return ExitCode::SUCCESS;
        }
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(2);
        }
    };

    // `try_get_one` separates "not supplied" (`Ok(None)`) from "supplied but
    // not a u16" (`Err`). `get_one` would merge both into a single `None`.
    let port = match matches.try_get_one::<u16>("port") {
        Ok(Some(port)) => port,
        Ok(None) => 8080,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(2);
        }
    };

    // Values already validated against `possible_values` during parsing, so a
    // plain `get_one` with the default as fallback is enough here.
    let mode: String = matches.get_one("mode").unwrap_or("safe".to_string());

    println!("Configuration loaded:");
    println!("  - Port:    {port}");
    println!("  - Mode:    {mode}");
    println!("  - Verbose: {}", matches.get_flag("verbose"));
    ExitCode::SUCCESS
}
