//! # Xarp
//!
//! A colorful and customizable command-line argument parser.

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
