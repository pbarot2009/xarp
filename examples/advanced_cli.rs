//! # Advanced CLI: Type-Safe Parsing & Multi-Values
//!
//! This example shows advanced usage patterns:
//! - Collecting repeated arguments with `ArgAction::Append` (e.g. `--header key:val --header key2:val2`).
//! - Parsing values directly into numeric types (`u16`, `usize`) and `PathBuf` via `FromArgValue`.
//! - Validating and handling execution flows across nested subcommands.
//! - Programmatic simulation of argument lists using `try_get_matches_from`.

use std::path::PathBuf;
use xarp::{Arg, ArgAction, Xarp};

/// Simulates a multi-faceted HTTP and deployment utility.
fn main() {
    let server_subcommand = Xarp::new("serve")
        .about("Spin up the local development HTTP server")
        // Type conversion: parsed directly into a numeric port (u16)
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("TCP port number to bind the server to")
                .default_value("8080"),
        )
        // Repeated option: users can supply `-H "Name: Value"` multiple times
        .arg(
            Arg::new("headers")
                .short('H')
                .long("header")
                .value_name("KEY:VAL")
                .help("Custom HTTP headers to inject into server responses")
                .action(ArgAction::Append),
        )
        // Direct PathBuf conversion
        .arg(
            Arg::new("root")
                .short('r')
                .long("root")
                .value_name("DIR")
                .help("Root directory for static assets")
                .default_value("."),
        );

    let cli = Xarp::new("nexus")
        .version("0.1.1-dev")
        .about("High-performance service runner and toolchain")
        .arg(
            Arg::new("workers")
                .short('w')
                .long("workers")
                .value_name("COUNT")
                .help("Number of asynchronous worker threads")
                .default_value("4"),
        )
        .subcommand(server_subcommand);

    // Demonstration: Parse from simulated string slices instead of env::args()
    let mock_args: Vec<String> = vec![
        "nexus".into(),
        "serve".into(),
        "--port".into(),
        "3000".into(),
        "-H".into(),
        "Cache-Control: no-cache".into(),
        "-H".into(),
        "X-Debug-Mode: true".into(),
        "--root".into(),
        "/var/www/html".into(),
    ];

    println!("Simulating input: {}\n", mock_args.join(" "));

    match cli.try_get_matches_from(&mock_args) {
        Ok(matches) => {
            if let Some(("serve", sub)) = matches.subcommand() {
                // Parse directly into u16
                let port: u16 = sub
                    .get_one("port")
                    .expect("Failed to parse port as a valid u16");

                // Parse into PathBuf
                let root: PathBuf = sub
                    .get_one("root")
                    .expect("Failed to parse root directory path");

                // Retrieve all entries supplied across multiple `--header` flags
                let headers: Vec<String> = sub.get_many("headers").unwrap_or_default();

                println!("Server configuration:");
                println!("  Listening on port : {port}");
                println!("  Document root     : {}", root.display());
                println!(
                    "  Injected headers  : ({} headers registered)",
                    headers.len()
                );
                for (idx, header) in headers.iter().enumerate() {
                    println!("    [{idx}] {header}");
                }
            }
        }
        Err(err) => {
            eprintln!("Command parsing failed:\n{err}");
        }
    }
}
