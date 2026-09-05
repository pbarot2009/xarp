# xarp

A zero-dependency, pure-Rust argument parser with rich 24-color and RGB terminal styling.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
xarp = "0.1.2-dev"
```

## Quick Start

```rust
use xarp::{Arg, ArgAction, Xarp};

fn main() {
    let matches = Xarp::new("demo")
        .version("0.1.2-dev")
        .about("A simple CLI built with xarp")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("PATH")
                .help("Configuration file path")
                .default_value("config.toml"),
        )
        .arg(
            Arg::new("input")
                .value_name("FILE")
                .help("Input file to process")
                .required(true),
        )
        .get_matches();

    let verbose: bool = matches.get_flag("verbose");
    let config: String = matches.get_one("config").unwrap();
    let input: String = matches.get_one("input").unwrap();

    println!("Verbose: {verbose}");
    println!("Config: {config}");
    println!("Input: {input}");
}
```

## Argument Types

 * Flags: Set `action(ArgAction::SetTrue)`. Checks via `matches.get_flag("id")`.
 * Options: Default action (`ArgAction::Set`). Value passed via `--opt value` or `--opt=value`.
   Repeated use keeps the last value; a following `--` is never consumed as a value.
 * Positional Arguments: Omit both `.short()` and `.long()`. Evaluated in declaration order.
   Only the last positional may use `ArgAction::Append` to collect multiple values.
 * Multiple Values: Use `action(ArgAction::Append)` and retrieve via `matches.get_many::<T>("id")`.

```rust
let matches = Xarp::new("app")
    .arg(
        Arg::new("tag")
            .short('t')
            .long("tag")
            .action(ArgAction::Append),
    )
    .get_matches();

if let Some(tags) = matches.get_many::<String>("tag") {
    for tag in tags {
        println!("Tag: {tag}");
    }
}
```

## Validation & Fallbacks

```rust
use xarp::{Arg, Xarp};

let matches = Xarp::new("app")
    .arg(
        Arg::new("mode")
            .long("mode")
            .value_name("MODE")
            .possible_values(["fast", "safe"])
            .default_value("fast"),
    )
    .arg(
        Arg::new("port")
            .long("port")
            .value_name("PORT")
            .env("APP_PORT"), // used when `--port` is not passed
    )
    .arg(
        Arg::new("json")
            .long("json")
            .conflicts_with("mode"), // error when combined with `--mode`
    )
    .get_matches();
```

Conflict checks consider explicitly passed arguments (CLI or environment),
never `default_value`s. A required positional must not follow an optional one.

## Typed Values & Errors

`get_one` returns `None` for both missing and unparsable values. Use
`try_get_one` / `try_get_many` to tell them apart:

```rust
let port: u16 = matches.try_get_one("port").unwrap().unwrap_or(8080);
```

Parse failures from `try_get_*` carry the same `--help` guidance as other
parse errors.

## Parsing Without Exiting

`get_matches()` prints help/version and exits the process. Library code
should use a `try_` variant and handle `XarpError` itself:

```rust
use xarp::{Arg, Xarp, XarpError};

let result = Xarp::new("app")
    .arg(Arg::new("verbose").short('v').long("verbose"))
    .try_get_matches();

match result {
    Ok(matches) => println!("verbose: {}", matches.get_flag("verbose")),
    Err(err) if err.is_help() || err.is_version() => print!("{err}"),
    Err(err) => eprintln!("{err}"),
}
```

`try_get_matches_with_env(&args, &env_map)` parses with an explicit
environment map instead of the process environment, which keeps tests
deterministic. Tokens after `--` are always positionals.

## Subcommands

Subcommands are nested `Xarp` instances:

```rust
use xarp::{Arg, Xarp};

fn main() {
    let commit = Xarp::new("commit")
        .about("Commit staged changes")
        .arg(Arg::new("message").short('m').long("message").required(true));

    let matches = Xarp::new("git-clone")
        .subcommand(commit)
        .get_matches();

    if let Some(("commit", sub_matches)) = matches.subcommand() {
        let message: String = sub_matches.get_one("message").unwrap();
        println!("Commit message: {message}");
    }
}
```

## Terminal Styling & Colors

`xarp` provides a self-contained ANSI coloring engine supporting 24 named palette colors, 8-bit ANSI 256, and 24-bit TrueColor (RGB).

```rust
use xarp::color::{Color, GREEN, RED};
use xarp::effect::Effects;
use xarp::style::Style;

// Using preset styles
println!("{}", GREEN.paint("Build succeeded"));
println!("{}", RED.paint("Build failed"));

// Custom builder styles
let warning = Style::new()
    .bold()
    .fg(Color::Yellow)
    .bg(Color::Black);

println!("{}", warning.paint("Disk usage above 90%"));

// TrueColor (RGB) and 256-color support
let rgb_style = Style::new().fg(Color::Rgb(255, 128, 0));
let ansi_style = Style::new().fg(Color::Ansi256(208));

// Operator overloading
let alert = Color::BrightRed | Effects::BOLD | Effects::UNDERLINE;
println!("{}", alert.paint("Fatal Error"));
```

Styles compose with `|` as well (`Style | Style` merges, right-hand colors
win). Help output follows the configured `Styles` theme; setting the
`NO_COLOR` environment variable (any value, including empty) disables colors.

## License
Dual-licensed under either of:
 * MIT License
 * Apache License, Version 2.0
