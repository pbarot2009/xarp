//! # Basic CLI Demo
//!
//! This example illustrates the core setup of a command-line interface using `xarp`.
//! It covers:
//! - Setting up application metadata (`name`, `version`, `about`).
//! - Declaring required positional arguments.
//! - Adding optional valued flags with default fallbacks.
//! - Configuring boolean toggle flags.
//! - Registering a basic subcommand and branching execution based on user input.

use xarp::{Arg, ArgAction, Xarp};

/// Entry point demonstrating basic parsing and value extraction.
fn main() {
    // 1. Configure the CLI application structure
    let app = Xarp::new("mycli")
        .version("0.1.1-dev")
        .about("A lightning-fast, styled CLI tool built with xarp")
        // Positional argument:
        // Positionals do not specify short or long flags.
        // They are evaluated sequentially in declaration order.
        .arg(
            Arg::new("input")
                .help("The input file to process")
                .value_name("FILE")
                .required(true),
        )
        // Boolean flag:
        // By setting `ArgAction::SetTrue`, the flag acts as a binary switch.
        // It does not require or accept an argument value (e.g. `-v` or `--verbose`).
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose debugging output")
                .action(ArgAction::SetTrue),
        )
        // Valued option with a default value:
        // If the user omits `-o` / `--output`, xarp automatically falls back
        // to the provided default string slice.
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Target path for generated output")
                .value_name("PATH")
                .default_value("dist/output.bin"),
        )
        // Subcommand declaration:
        // Subcommands are isolated `Xarp` instances with their own distinct arguments.
        .subcommand(
            Xarp::new("build")
                .about("Compile source packages into binary distributions")
                .arg(
                    Arg::new("release")
                        .long("release")
                        .help("Compile artifacts in release mode with optimizations")
                        .action(ArgAction::SetTrue),
                ),
        );

    // 2. Parse process arguments from `std::env::args()`
    // `get_matches()` automatically prints styled help text on `-h`/`--help`
    // and exits with status 2 on syntax or validation errors.
    let matches = app.get_matches();

    // 3. Handle subcommands
    // `subcommand()` returns a tuple containing the subcommand name and its inner matches.
    if let Some(("build", sub_matches)) = matches.subcommand() {
        let is_release = sub_matches.get_flag("release");
        println!("==> Subcommand: build");
        println!("    Optimized release build: {is_release}");
        return;
    }

    // 4. Extract top-level matches
    // `get_one::<T>` parses the raw CLI string into any type implementing `FromStr`.
    let input: String = matches
        .get_one("input")
        .expect("Argument 'input' is required");

    let output: String = matches
        .get_one("output")
        .expect("Argument 'output' has a default value");

    let is_verbose: bool = matches.get_flag("verbose");

    println!("Configuration loaded:");
    println!("  - Input:   {input}");
    println!("  - Output:  {output}");
    println!("  - Verbose: {is_verbose}");
}
