//! # Git-Style Nested Subcommands
//!
//! This example builds a miniature version-control CLI with `xarp`:
//! - Nested subcommands: `commit` and `remote` with its own `add` / `remove`
//!   children (`vcs remote add <name> <url>`).
//! - Parent flags such as `--verbose` supplied before the subcommand.
//! - Parent defaults (here `--format`) that stay available in subcommand mode.
//! - Option values may start with `-` and are still accepted: a commit message
//!   of `-m` is written as `vcs commit --message -m`.
//! - The `--` delimiter forces positional parsing, which disambiguates values
//!   equal to a subcommand name.
//!
//! Try it with:
//! - `cargo run -q --example git_style_subcommands -- commit -m "initial"`
//! - `cargo run -q --example git_style_subcommands -- --verbose remote add origin https://example.com/repo`
//! - `cargo run -q --example git_style_subcommands -- remote remove origin`
//! - `cargo run -q --example git_style_subcommands -- commit --message -m`

use std::process::ExitCode;
use xarp::{Arg, ArgAction, Xarp};

/// Builds the CLI. A clone is kept for the bare-invocation help screen
/// because parsing consumes the definition.
fn build_app() -> Xarp {
    let commit = Xarp::new("commit").about("Record a new commit").arg(
        Arg::new("message")
            .short('m')
            .long("message")
            .value_name("MSG")
            .help("Commit message (dash-leading values such as `-m` are accepted)")
            .required(true),
    );

    let remote_add = Xarp::new("add")
        .about("Register a new remote")
        .arg(
            Arg::new("name")
                .value_name("NAME")
                .help("Remote name")
                .required(true),
        )
        .arg(
            Arg::new("url")
                .value_name("URL")
                .help("Remote URL")
                .required(true),
        );
    let remote_remove = Xarp::new("remove").about("Forget a remote").arg(
        Arg::new("name")
            .value_name("NAME")
            .help("Remote name")
            .required(true),
    );
    let remote = Xarp::new("remote")
        .about("Manage remotes")
        .subcommand(remote_add)
        .subcommand(remote_remove);

    Xarp::new("vcs")
        .version("0.1.2-dev")
        .about("A tiny version-control CLI built with xarp")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_name("FORMAT")
                .help("Output format")
                .possible_values(["short", "full"])
                .default_value("short"),
        )
        .subcommand(commit)
        .subcommand(remote)
}

/// Handles the `remote` subcommand and its `add` / `remove` children.
fn run_remote(matches: &xarp::ArgMatches, verbose: bool, format: &str) -> ExitCode {
    match matches.subcommand() {
        Some(("add", sub)) => {
            let name: String = sub.get_one("name").unwrap_or("<missing>".to_string());
            let url: String = sub.get_one("url").unwrap_or("<missing>".to_string());
            println!("Adding remote [{format}]: {name} -> {url} (verbose: {verbose})");
            ExitCode::SUCCESS
        }
        Some(("remove", sub)) => {
            let name: String = sub.get_one("name").unwrap_or("<missing>".to_string());
            println!("Removing remote [{format}]: {name} (verbose: {verbose})");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("Usage: vcs remote (add | remove) ...");
            ExitCode::from(2)
        }
    }
}

/// Entry point: parse once, then dispatch on the matched subcommand.
fn main() -> ExitCode {
    let app = build_app();
    let help_app = app.clone();
    let matches = match app.try_get_matches() {
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

    // Parent selections work in subcommand mode: flags set before the
    // subcommand are kept and defaults are still applied.
    let verbose = matches.get_flag("verbose");
    let format: String = matches.get_one("format").unwrap_or("short".to_string());

    match matches.subcommand() {
        Some(("commit", sub)) => {
            let message: String = sub.get_one("message").unwrap_or("<missing>".to_string());
            println!("Committing [{format}]: {message} (verbose: {verbose})");
            ExitCode::SUCCESS
        }
        Some(("remote", sub)) => run_remote(sub, verbose, &format),
        _ => {
            // No subcommand: show the top-level help instead of failing with
            // a bare usage error.
            help_app.print_help();
            ExitCode::SUCCESS
        }
    }
}
