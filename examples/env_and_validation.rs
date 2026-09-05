//! # Environment Fallbacks & Validation
//!
//! This example shows how `xarp` resolves configuration from three sources:
//! - Explicit CLI flags, which always win.
//! - `.env("VAR")` fallbacks, used when the flag is absent.
//! - `.default_value(...)`, used when neither the flag nor the environment
//!   provides a value.
//!
//! It also demonstrates validation:
//! - `possible_values` allow-lists, checked for CLI, environment, and default
//!   values alike.
//! - `conflicts_with` mutual exclusion, which only considers explicitly
//!   supplied arguments (defaults never conflict).
//!
//! Try it with:
//! - `cargo run -q --example env_and_validation`
//! - `cargo run -q --example env_and_validation -- --region eu --json`
//! - `APP_REGION=asia APP_WORKERS=8 cargo run -q --example env_and_validation`
//! - `cargo run -q --example env_and_validation -- --json --yaml` (conflict)

use std::process::ExitCode;
use xarp::{Arg, ArgAction, Xarp};

/// Builds the deploy-tool configuration shared by every run below.
fn build_app() -> Xarp {
    Xarp::new("deployer")
        .version("0.1.2-dev")
        .about("Reads deployment settings from flags, environment, and defaults")
        .arg(
            Arg::new("region")
                .long("region")
                .value_name("REGION")
                .help("Target region")
                .possible_values(["us", "eu", "asia"])
                .env("APP_REGION")
                .default_value("us"),
        )
        .arg(
            Arg::new("workers")
                .short('w')
                .long("workers")
                .value_name("COUNT")
                .help("Worker thread count")
                .env("APP_WORKERS")
                .default_value("4"),
        )
        // Output formats are mutually exclusive, but only when the user
        // explicitly selects them: defaults never trigger a conflict.
        .arg(
            Arg::new("json")
                .long("json")
                .help("Emit JSON output")
                .action(ArgAction::SetTrue)
                .conflicts_with("yaml"),
        )
        .arg(
            Arg::new("yaml")
                .long("yaml")
                .help("Emit YAML output")
                .action(ArgAction::SetTrue),
        )
}

/// Resolves the configuration and prints where each value came from.
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

    // The region was already checked against `possible_values` while parsing.
    let region: String = matches.get_one("region").unwrap_or("us".to_string());

    // Worker count parsing can still fail (e.g. `APP_WORKERS=lots`), so the
    // fallible getter reports the offending value instead of hiding it.
    let workers = match matches.try_get_one::<u16>("workers") {
        Ok(Some(count)) => count,
        Ok(None) => 4,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(2);
        }
    };

    let format = if matches.get_flag("json") {
        "json"
    } else if matches.get_flag("yaml") {
        "yaml"
    } else {
        "text"
    };

    println!("Resolved configuration:");
    println!("  - Region:  {region}");
    println!("  - Workers: {workers}");
    println!("  - Format:  {format}");
    ExitCode::SUCCESS
}
