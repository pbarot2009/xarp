//! # Xarp
//!
//! A zero-dependency, pure-Rust command-line argument parser with a
//! self-contained ANSI styling engine (24 named colors, 8-bit 256-color,
//! and 24-bit true color).
//!
//! ## Quick start
//!
//! ```rust
//! use xarp::{Arg, ArgAction, Xarp};
//!
//! let matches = Xarp::new("demo")
//!     .version("0.1.2-dev")
//!     .about("A simple CLI built with xarp")
//!     .arg(
//!         Arg::new("verbose")
//!             .short('v')
//!             .long("verbose")
//!             .help("Enable verbose output")
//!             .action(ArgAction::SetTrue),
//!     )
//!     .arg(Arg::new("input").value_name("FILE").required(true))
//!     .try_get_matches_from(&["demo".to_string(), "file.txt".to_string()])
//!     .unwrap();
//!
//! assert!(!matches.get_flag("verbose"));
//! assert_eq!(
//!     matches.get_one::<String>("input"),
//!     Some("file.txt".to_string())
//! );
//! ```
//!
//! Binaries normally finish with [`Xarp::get_matches`], which prints help or
//! version and exits the process on failure; library code prefers the
//! `try_` variants, which return [`XarpError`] instead.
//!
//! ## Parsing model
//!
//! - Flags use [`ArgAction::SetTrue`] and are read with
//!   [`ArgMatches::get_flag`].
//! - Options use [`ArgAction::Set`] (last occurrence wins) and accept
//!   `--opt value`, `--opt=value`, `-p value`, and `-pvalue`.
//! - [`ArgAction::Append`] options (and a trailing positional) collect every
//!   occurrence, read with [`ArgMatches::get_many`].
//! - Arguments without [`Arg::short`] or [`Arg::long`] are positionals,
//!   matched in declaration order; tokens after `--` are always positionals.
//! - Subcommands are nested [`Xarp`] parsers; a value equal to a subcommand
//!   name can still be passed as a positional behind `--`.
//!
//! ## Validation and fallbacks
//!
//! Arguments support [`Arg::required`], [`Arg::possible_values`],
//! [`Arg::conflicts_with`], [`Arg::default_value`], and [`Arg::env`]
//! fallbacks with CLI-beats-environment-beats-default precedence. Conflict
//! checks only consider explicitly supplied arguments, never defaults.
//! Typed access that distinguishes missing values from invalid ones is
//! available via [`ArgMatches::try_get_one`] and
//! [`ArgMatches::try_get_many`].
//!
//! ## Terminal styling
//!
//! [`Style`] combines a foreground [`Color`], a background color, and
//! [`Effects`] bitflags, renders ANSI escapes via [`Display`][core::fmt::Display],
//! and paints values with [`Style::paint`]. [`Styles`] themes the help,
//! version, and error output; setting `NO_COLOR` (any value, including empty)
//! disables colors.
//!
//! ## Modules
//!
//! - [`color`]: color definitions and preset styles.
//! - [`effect`]: text effect bitflags.
//! - [`style`]: styles, painting, and CLI themes.
//! - [`xarp`]: argument definitions, parsing, and matches.
//!
//! ## Examples
//!
//! The repository ships runnable examples: `basic_demo`, `advanced_cli`,
//! `styling_and_theming`, `error_handling`, `env_and_validation`, and
//! `git_style_subcommands` (see `examples/`).

/// Color definitions, ANSI escape sequence generation, and color style presets.
pub mod color;

/// ANSI text style modifiers and font effects bitflags.
pub mod effect;

/// Text styles, ANSI output formatting, and CLI theme configurations.
pub mod style;

/// Core command-line argument parsing and execution engine.
pub mod xarp;

pub use color::Color;
pub use effect::Effects;
pub use style::{Style, Styled, Styles};
pub use xarp::{Arg, ArgAction, ArgMatches, FromArgValue, Xarp, XarpError};
