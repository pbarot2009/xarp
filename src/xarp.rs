use crate::style::{Style, Styles};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write;
use std::process;

use std::error::Error;
use std::fmt::{self, Display};

/// Errors encountered while evaluating command-line arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum XarpError {
    /// Help documentation requested by the user.
    Help(String),
    /// Version details requested by the user.
    Version(String),
    /// Argument syntax or validation failure.
    Parse(String),
}

impl Display for XarpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Help(msg) | Self::Version(msg) | Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl XarpError {
    /// Returns `true` when the user requested help output.
    ///
    /// The payload already contains the rendered help text.
    #[must_use]
    pub fn is_help(&self) -> bool {
        matches!(self, Self::Help(_))
    }

    /// Returns `true` when the user requested version output.
    #[must_use]
    pub fn is_version(&self) -> bool {
        matches!(self, Self::Version(_))
    }

    /// Returns `true` for argument syntax or validation failures.
    #[must_use]
    pub fn is_parse(&self) -> bool {
        matches!(self, Self::Parse(_))
    }
}

impl Error for XarpError {}

impl From<String> for XarpError {
    fn from(msg: String) -> Self {
        Self::Parse(msg)
    }
}

/// Specifies the parsing action to take when an argument is encountered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgAction {
    /// Flag without a value (e.g., `-v`, `--verbose`). Sets the flag to `true`.
    SetTrue,
    /// Option or positional argument that captures a single value.
    Set,
    /// Option that can be supplied multiple times to collect values into a list.
    Append,
}

/// Represents a command-line argument definition.
#[derive(Clone, Debug)]
pub struct Arg {
    /// Unique identifier for the argument.
    pub id: &'static str,
    /// Short single-character flag (e.g., `'v'` for `-v`).
    pub short: Option<char>,
    /// Long flag name (e.g., `"verbose"` for `--verbose`).
    pub long: Option<&'static str>,
    /// Description of the argument displayed in help messages.
    pub help: Option<&'static str>,
    /// Placeholder name for the argument value displayed in help messages.
    pub value_name: Option<&'static str>,
    /// Whether the argument must be supplied by the user.
    pub required: bool,
    /// Parsing action performed when this argument is encountered.
    pub action: ArgAction,
    /// Fallback value used when the argument is not explicitly provided.
    pub default_value: Option<&'static str>,
    /// Environment variable used when the argument is not explicitly passed.
    pub env: Option<&'static str>,
    /// Allowed input choices for the argument value.
    pub possible_values: Vec<&'static str>,
    /// IDs of other arguments that cannot be supplied alongside this one.
    pub conflicts_with: Vec<&'static str>,
}

impl Arg {
    /// Creates a new argument definition with the specified identifier.
    #[must_use]
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            short: None,
            long: None,
            help: None,
            value_name: None,
            required: false,
            action: ArgAction::Set,
            default_value: None,
            env: None,
            possible_values: Vec::new(),
            conflicts_with: Vec::new(),
        }
    }

    /// Restricts argument values to a defined set of choices.
    #[must_use]
    pub fn possible_values<I: IntoIterator<Item = &'static str>>(mut self, values: I) -> Self {
        self.possible_values = values.into_iter().collect();
        self
    }

    /// Binds an environment variable fallback to the argument.
    #[must_use]
    pub fn env(mut self, env_var: &'static str) -> Self {
        self.env = Some(env_var);
        self
    }

    /// Marks an argument identifier as mutually exclusive with this argument.
    #[must_use]
    pub fn conflicts_with(mut self, other_id: &'static str) -> Self {
        self.conflicts_with.push(other_id);
        self
    }

    /// Sets the short character flag.
    #[must_use]
    pub fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Sets the long flag name.
    #[must_use]
    pub fn long(mut self, long: &'static str) -> Self {
        self.long = Some(long);
        self
    }

    /// Sets the help message description.
    #[must_use]
    pub fn help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }

    /// Sets the placeholder value name for help messages.
    #[must_use]
    pub fn value_name(mut self, name: &'static str) -> Self {
        self.value_name = Some(name);
        self
    }

    /// Sets whether the argument is required.
    #[must_use]
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets the argument evaluation action.
    #[must_use]
    pub fn action(mut self, action: ArgAction) -> Self {
        self.action = action;
        self
    }

    /// Sets the fallback default value.
    #[must_use]
    pub fn default_value(mut self, default: &'static str) -> Self {
        self.default_value = Some(default);
        self
    }

    /// Returns `true` if the argument has neither short nor long flags.
    #[must_use]
    #[inline]
    pub fn is_positional(&self) -> bool {
        self.short.is_none() && self.long.is_none()
    }
}

/// Container holding parsed command-line flags, values, and subcommands.
#[derive(Clone, Debug, Default)]
pub struct ArgMatches {
    flags: HashSet<String>,
    values: HashMap<String, Vec<String>>,
    subcommand: Option<(String, Box<ArgMatches>)>,
}

impl ArgMatches {
    /// Checks whether a boolean flag was supplied.
    #[must_use]
    pub fn get_flag(&self, id: &str) -> bool {
        self.flags.contains(id)
    }

    /// Gets the first or single value for an argument, parsed into `T`.
    ///
    /// Returns `None` both when the argument is absent and when parsing fails.
    /// Use [`ArgMatches::try_get_one`] to distinguish the two cases.
    #[must_use]
    pub fn get_one<T: FromArgValue>(&self, id: &str) -> Option<T> {
        self.values
            .get(id)
            .and_then(|vals| vals.first())
            .and_then(|val| T::from_arg_value(val))
    }

    /// Gets all values supplied for an argument, parsed into `Vec<T>`.
    ///
    /// Values that fail to parse are silently skipped, so a present argument
    /// whose values all fail to parse yields `Some(vec![])`.
    /// Use [`ArgMatches::try_get_many`] to surface parse failures as errors.
    #[must_use]
    pub fn get_many<T: FromArgValue>(&self, id: &str) -> Option<Vec<T>> {
        self.values
            .get(id)
            .map(|vals| vals.iter().filter_map(|v| T::from_arg_value(v)).collect())
    }

    /// Hint appended to [`ArgMatches::try_get_one`] and
    /// [`ArgMatches::try_get_many`] failures so they match the
    /// `--help` guidance of other parse errors.
    const TRY_HELP_HINT: &'static str = "\n\nFor more information, try '--help'.";

    /// Gets the first or single value for an argument, distinguishing
    /// missing arguments from parse failures.
    ///
    /// * `Ok(None)` — the argument was not supplied.
    /// * `Ok(Some(v))` — the argument was supplied and parsed successfully.
    /// * `Err(_)` — the argument was supplied but its value failed to parse into `T`.
    ///
    /// # Errors
    ///
    /// Returns [`XarpError::Parse`] if a supplied value cannot be parsed into `T`.
    /// The message carries the same `--help` guidance as other parse errors.
    pub fn try_get_one<T: FromArgValue>(&self, id: &str) -> Result<Option<T>, XarpError> {
        match self.values.get(id).and_then(|vals| vals.first()) {
            None => Ok(None),
            Some(raw) => match T::from_arg_value(raw) {
                Some(parsed) => Ok(Some(parsed)),
                None => Err(XarpError::Parse(format!(
                    "invalid value '{raw}' for '{id}'{}",
                    Self::TRY_HELP_HINT
                ))),
            },
        }
    }

    /// Gets all values supplied for an argument, surfacing parse failures.
    ///
    /// Returns `Ok(None)` when the argument was not supplied.
    ///
    /// # Errors
    ///
    /// Returns [`XarpError::Parse`] if any supplied value cannot be parsed into `T`.
    /// The message carries the same `--help` guidance as other parse errors.
    pub fn try_get_many<T: FromArgValue>(&self, id: &str) -> Result<Option<Vec<T>>, XarpError> {
        match self.values.get(id) {
            None => Ok(None),
            Some(vals) => {
                let mut out = Vec::with_capacity(vals.len());
                for raw in vals {
                    match T::from_arg_value(raw) {
                        Some(parsed) => out.push(parsed),
                        None => {
                            return Err(XarpError::Parse(format!(
                                "invalid value '{raw}' for '{id}'{}",
                                Self::TRY_HELP_HINT
                            )));
                        }
                    }
                }
                Ok(Some(out))
            }
        }
    }

    /// Returns the matched subcommand name and its parsed matches, if present.
    #[must_use]
    pub fn subcommand(&self) -> Option<(&str, &ArgMatches)> {
        self.subcommand
            .as_ref()
            .map(|(name, matches)| (name.as_str(), matches.as_ref()))
    }
}

/// Helper trait to parse argument strings into typed values.
pub trait FromArgValue: Sized {
    /// Parses a string slice into `Self`, returning `None` if parsing fails.
    fn from_arg_value(val: &str) -> Option<Self>;
}

impl<T: std::str::FromStr> FromArgValue for T {
    fn from_arg_value(val: &str) -> Option<Self> {
        val.parse().ok()
    }
}

/// Command-line argument parser and application configuration.
#[derive(Clone, Debug)]
pub struct Xarp {
    /// Name of the application binary.
    pub name: &'static str,
    /// Version string displayed in help and version outputs.
    pub version: Option<&'static str>,
    /// Short description of the application.
    pub about: Option<&'static str>,
    /// Formatting styles applied to terminal outputs.
    pub styles: Styles,
    /// Registered argument definitions.
    pub args: Vec<Arg>,
    /// Registered subcommands.
    pub subcommands: Vec<Xarp>,
}

impl Xarp {
    /// Creates a new command-line application definition.
    #[must_use]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            version: None,
            about: None,
            styles: Styles::styled(),
            args: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    /// Sets the application version string.
    #[must_use]
    pub fn version(mut self, version: &'static str) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets the short description of the application.
    #[must_use]
    pub fn about(mut self, about: &'static str) -> Self {
        self.about = Some(about);
        self
    }

    /// Sets the terminal output styling.
    #[must_use]
    pub fn styles(mut self, styles: Styles) -> Self {
        self.styles = styles;
        self
    }

    /// Registers a single argument definition.
    #[must_use]
    pub fn arg(mut self, arg: Arg) -> Self {
        self.args.push(arg);
        self
    }

    /// Registers multiple argument definitions from an iterator.
    #[must_use]
    pub fn args<I: IntoIterator<Item = Arg>>(mut self, args: I) -> Self {
        self.args.extend(args);
        self
    }

    /// Registers a subcommand.
    #[must_use]
    pub fn subcommand(mut self, sub: Xarp) -> Self {
        self.subcommands.push(sub);
        self
    }

    /// Parses arguments from `std::env::args()` or terminates the process on failure.
    ///
    /// This is a convenience wrapper for binaries: help prints to standard
    /// output and exits with status `0`, version does the same, and parse
    /// failures print to standard error and exit with status `2`. Library code
    /// and tests should prefer [`Xarp::try_get_matches`], which returns the
    /// [`XarpError`] instead of exiting.
    #[must_use]
    pub fn get_matches(self) -> ArgMatches {
        match self.try_get_matches() {
            Ok(matches) => matches,
            Err(XarpError::Help(msg)) => {
                print!("{msg}");
                process::exit(0);
            }
            Err(XarpError::Version(msg)) => {
                println!("{msg}");
                process::exit(0);
            }
            Err(XarpError::Parse(err)) => {
                eprintln!("{err}");
                process::exit(2);
            }
        }
    }

    /// Parses arguments from `std::env::args()` without exiting on error.
    ///
    /// Unlike [`Xarp::get_matches`], help, version, and parse failures are
    /// returned as [`XarpError`] so callers decide how to report them.
    ///
    /// # Errors
    ///
    /// Returns [`XarpError`] if parsing fails or when `--help`/`--version` flags are passed.
    pub fn try_get_matches(self) -> Result<ArgMatches, XarpError> {
        let args: Vec<String> = env::args().collect();
        self.try_get_matches_from(&args)
    }

    /// Returns `true` for truthy flag values (`"1"` or case-insensitive `"true"`).
    fn is_truthy(val: &str) -> bool {
        val == "1" || val.eq_ignore_ascii_case("true")
    }

    /// Validates argument definitions: duplicate ids, shorts, longs and
    /// duplicate subcommand names.
    ///
    /// Also rejects empty or ambiguous definitions (empty ids, reserved short
    /// flags, malformed longs, unreachable subcommand names), unknown
    /// [`Arg::conflicts_with`] targets, and optional positionals declared
    /// before required ones.
    fn validate_definitions(&self) -> Result<(), XarpError> {
        self.validate_names()?;
        self.validate_flags()?;
        self.validate_relations()?;
        Ok(())
    }

    /// Rejects empty application/argument ids and unreachable subcommand names.
    fn validate_names(&self) -> Result<(), XarpError> {
        if self.name.is_empty() {
            return Err(XarpError::Parse(
                self.format_error("application name must not be empty"),
            ));
        }
        let mut ids = HashSet::new();
        for arg in &self.args {
            if arg.id.is_empty() {
                return Err(XarpError::Parse(
                    self.format_error("argument id must not be empty"),
                ));
            }
            if !ids.insert(arg.id) {
                return Err(XarpError::Parse(self.format_error(&format!(
                    "duplicate argument id '{}' found",
                    arg.id
                ))));
            }
        }
        let mut sub_names = HashSet::new();
        for sub in &self.subcommands {
            if sub.name.is_empty() || sub.name.starts_with('-') {
                return Err(XarpError::Parse(self.format_error(&format!(
                    "invalid subcommand name '{}': must be non-empty and not start with '-'",
                    sub.name
                ))));
            }
            if !sub_names.insert(sub.name) {
                return Err(XarpError::Parse(self.format_error(&format!(
                    "duplicate subcommand '{0}' found",
                    sub.name
                ))));
            }
        }
        Ok(())
    }

    /// Rejects duplicate or malformed short and long flags.
    fn validate_flags(&self) -> Result<(), XarpError> {
        let mut shorts = HashSet::new();
        for arg in &self.args {
            if let Some(short) = arg.short {
                if short == '-' || short == '=' || short.is_whitespace() || short.is_control() {
                    return Err(XarpError::Parse(self.format_error(&format!(
                        "invalid short flag '-{short}' for '{}': must not be '-', '=' or whitespace",
                        arg.id
                    ))));
                }
                if !shorts.insert(short) {
                    return Err(XarpError::Parse(
                        self.format_error(&format!("duplicate short flag '-{short}' found")),
                    ));
                }
            }
        }
        let mut longs = HashSet::new();
        for arg in &self.args {
            if let Some(long) = arg.long {
                if long.is_empty()
                    || long.starts_with('-')
                    || long.contains('=')
                    || long.chars().any(|c| c.is_whitespace() || c.is_control())
                {
                    return Err(XarpError::Parse(self.format_error(&format!(
                        "invalid long flag '--{long}' for '{}'",
                        arg.id
                    ))));
                }
                if !longs.insert(long) {
                    return Err(XarpError::Parse(
                        self.format_error(&format!("duplicate long flag '--{long}' found")),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Validates relations between definitions: conflict targets must exist
    /// and required positionals must precede optional ones.
    fn validate_relations(&self) -> Result<(), XarpError> {
        // Conflict targets must name a defined argument (or an auto-injected
        // built-in) so typos cannot silently disable the check.
        let ids: HashSet<&'static str> = self.args.iter().map(|a| a.id).collect();
        let has_help_collision = self
            .args
            .iter()
            .any(|a| a.id == "help" || a.short == Some('h') || a.long == Some("help"));
        let has_version_collision = self
            .args
            .iter()
            .any(|a| a.id == "version" || a.short == Some('V') || a.long == Some("version"));
        for arg in &self.args {
            for conflict_id in &arg.conflicts_with {
                let known = ids.contains(conflict_id)
                    || (!has_help_collision && *conflict_id == "help")
                    || (self.version.is_some()
                        && !has_version_collision
                        && *conflict_id == "version");
                if !known {
                    return Err(XarpError::Parse(self.format_error(&format!(
                        "the argument '{}' conflicts with unknown argument '{conflict_id}'",
                        arg.id
                    ))));
                }
            }
        }
        // A required positional after an optional one can never be reached by
        // skipping the optional slot, so reject the definition.
        let mut seen_optional_positional = false;
        for arg in self.args.iter().filter(|a| a.is_positional()) {
            if arg.required {
                if seen_optional_positional {
                    return Err(XarpError::Parse(self.format_error(&format!(
                        "required positional '{}' must not follow an optional positional",
                        arg.id
                    ))));
                }
            } else {
                seen_optional_positional = true;
            }
        }
        Ok(())
    }

    /// Applies environment fallbacks and default values.
    ///
    /// Arguments already present from CLI parsing are left untouched.
    /// Values coming from the environment count as explicit (they satisfy
    /// `required` and participate in conflict detection); `default_value`s
    /// do not. A `SetTrue` default only sets the flag when truthy.
    /// An explicitly falsy environment value for `SetTrue` overrides any
    /// default and (when `skip_required` is false) still enforces `required`.
    #[allow(clippy::too_many_lines)]
    fn apply_env_defaults(
        &self,
        effective_args: &[Arg],
        matches: &mut ArgMatches,
        explicit: &mut HashSet<String>,
        env_lookup: &dyn Fn(&str) -> Option<String>,
        skip_required: bool,
    ) -> Result<(), XarpError> {
        for arg in effective_args {
            if matches.values.contains_key(arg.id) || matches.flags.contains(arg.id) {
                continue;
            }
            if let Some(env_key) = arg.env {
                if let Some(env_val) = env_lookup(env_key) {
                    if arg.action == ArgAction::SetTrue {
                        if Self::is_truthy(&env_val) {
                            matches.flags.insert(arg.id.to_string());
                            explicit.insert(arg.id.to_string());
                        } else if arg.required && !skip_required {
                            return Err(XarpError::Parse(self.format_error(&format!(
                                "the required argument '{}' was not provided",
                                arg.id
                            ))));
                        }
                        // Falsy env overrides any default: do not fall through.
                        continue;
                    }
                    matches.values.insert(arg.id.to_string(), vec![env_val]);
                    explicit.insert(arg.id.to_string());
                    continue;
                }
                // Env var unset: fall through to defaults/required.
            }
            if let Some(default) = arg.default_value {
                if arg.action == ArgAction::SetTrue {
                    if Self::is_truthy(default) {
                        matches.flags.insert(arg.id.to_string());
                    }
                } else {
                    matches
                        .values
                        .insert(arg.id.to_string(), vec![default.to_string()]);
                }
            } else if arg.required && !skip_required {
                return Err(XarpError::Parse(self.format_error(&format!(
                    "the required argument '{}' was not provided",
                    arg.id
                ))));
            }
        }
        Ok(())
    }

    /// Validates `possible_values` (including defaults/env) and
    /// `conflicts_with` (explicit CLI/env selections only, never defaults).
    fn validate_values(
        &self,
        effective_args: &[Arg],
        matches: &ArgMatches,
        explicit: &HashSet<String>,
    ) -> Result<(), XarpError> {
        for arg in effective_args {
            if !arg.possible_values.is_empty() {
                if let Some(vals) = matches.values.get(arg.id) {
                    for v in vals {
                        if !arg.possible_values.contains(&v.as_str()) {
                            let allowed = arg.possible_values.join(", ");
                            return Err(XarpError::Parse(self.format_error(&format!(
                                "invalid value '{v}' for '{}'. [possible values: {allowed}]",
                                arg.id
                            ))));
                        }
                    }
                }
            }
        }
        for arg in effective_args {
            if explicit.contains(arg.id) {
                for conflict_id in &arg.conflicts_with {
                    if explicit.contains(*conflict_id) {
                        return Err(XarpError::Parse(self.format_error(&format!(
                            "the argument '{}' cannot be used with '{}'",
                            arg.id, conflict_id
                        ))));
                    }
                }
            }
        }
        Ok(())
    }

    /// Attempts to parse arguments from a string slice without exiting on error.
    ///
    /// Environment fallbacks are read from the real process environment. For
    /// deterministic parsing in tests, use
    /// [`Xarp::try_get_matches_with_env`] with an explicit map instead.
    ///
    /// The first element is treated as the program name and skipped. An option
    /// that expects a value takes it from `--opt=value`, an attached short
    /// value (`-p8080`), or the next token; a following `--` delimiter is
    /// never consumed as a value and instead reports a missing value. Tokens
    /// after a `--` delimiter are always positionals, which also lets callers
    /// pass positional values equal to a subcommand name.
    ///
    /// # Errors
    ///
    /// Returns [`XarpError`] if parsing fails or when `--help`/`--version` flags are passed.
    #[allow(clippy::too_many_lines)]
    pub fn try_get_matches_from(self, args: &[String]) -> Result<ArgMatches, XarpError> {
        self.parse_impl(args, &|key| env::var(key).ok())
    }

    /// Attempts to parse arguments using an explicit environment map.
    ///
    /// Behaves exactly like [`Xarp::try_get_matches_from`], except `env`
    /// fallbacks are looked up in `env_map` instead of the process
    /// environment, making parsing fully deterministic under test.
    ///
    /// # Errors
    ///
    /// Returns [`XarpError`] if parsing fails or when `--help`/`--version` flags are passed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use xarp::{Arg, Xarp};
    ///
    /// let app = Xarp::new("demo").arg(Arg::new("out").long("out").env("OUT"));
    /// let env_map = HashMap::from([("OUT".to_string(), "file.txt".to_string())]);
    /// let matches = app
    ///     .try_get_matches_with_env(&["demo".to_string()], &env_map)
    ///     .unwrap();
    /// let out: String = matches.get_one("out").unwrap();
    /// assert_eq!(out, "file.txt");
    /// ```
    pub fn try_get_matches_with_env(
        self,
        args: &[String],
        env_map: &HashMap<String, String>,
    ) -> Result<ArgMatches, XarpError> {
        self.parse_impl(args, &|key| env_map.get(key).cloned())
    }

    /// Shared parsing implementation parameterized over the environment source.
    #[allow(clippy::too_many_lines)]
    fn parse_impl(
        self,
        args: &[String],
        env_lookup: &dyn Fn(&str) -> Option<String>,
    ) -> Result<ArgMatches, XarpError> {
        self.validate_definitions()?;
        let mut matches = ArgMatches::default();
        // Tracks arguments explicitly selected via CLI or environment.
        // Defaults never count as explicit (see `apply_env_defaults`).
        let mut explicit: HashSet<String> = HashSet::new();
        let tokens = if args.is_empty() { &[] } else { &args[1..] };

        // Inject default built-in help and version flags, avoiding collisions
        // with user-defined ids, shorts or longs.
        let mut effective_args = self.args.clone();
        let has_help_collision = effective_args
            .iter()
            .any(|a| a.id == "help" || a.short == Some('h') || a.long == Some("help"));
        if !has_help_collision {
            effective_args.push(
                Arg::new("help")
                    .short('h')
                    .long("help")
                    .help("Print help information")
                    .action(ArgAction::SetTrue),
            );
        }
        let has_version_collision = effective_args
            .iter()
            .any(|a| a.id == "version" || a.short == Some('V') || a.long == Some("version"));
        if self.version.is_some() && !has_version_collision {
            effective_args.push(
                Arg::new("version")
                    .short('V')
                    .long("version")
                    .help("Print version information")
                    .action(ArgAction::SetTrue),
            );
        }
        // Whether the built-in help/version handlers are active. When the user
        // defines a colliding id/short/long we skip injection and treat
        // `-h`/`--help`/`-V`/`--version` as ordinary arguments (no hijack).
        let help_injected = !has_help_collision;
        let version_injected = self.version.is_some() && !has_version_collision;

        let mut positional_idx = 0;
        let positional_args: Vec<&Arg> = effective_args
            .iter()
            .filter(|a| a.is_positional())
            .collect();
        // Only the last positional may collect multiple values.
        if positional_args.len() > 1 {
            for (idx, pos) in positional_args.iter().enumerate() {
                if pos.action == ArgAction::Append && idx + 1 != positional_args.len() {
                    return Err(XarpError::Parse(self.format_error(&format!(
                        "the argument '{}' with Append action must be the last positional",
                        pos.id
                    ))));
                }
            }
        }

        let mut i = 0;
        let mut only_positionals = false;

        while i < tokens.len() {
            let token = &tokens[i];

            // End of options delimiter: everything after '--' is a positional argument
            if token == "--" && !only_positionals {
                only_positionals = true;
                i += 1;
                continue;
            }

            // Subcommands take precedence over positionals. A positional value
            // equal to a subcommand name can still be forced with `--`
            // (which sets `only_positionals` and skips this branch).
            if !only_positionals
                && !token.starts_with('-')
                && positional_idx == 0
                && let Some(sub) = self.subcommands.iter().find(|s| s.name == token)
            {
                // Finalize parent selections (defaults/env + validation) so
                // `get_one` defaults remain available in subcommand mode.
                // Parent `required` args are skipped: the subcommand is an
                // alternative command, not an extension of the parent.
                self.apply_env_defaults(
                    &effective_args,
                    &mut matches,
                    &mut explicit,
                    env_lookup,
                    true,
                )?;
                self.validate_values(&effective_args, &matches, &explicit)?;
                let sub_matches = sub.clone().try_get_matches_from(&tokens[i..])?;
                matches.subcommand = Some((sub.name.to_string(), Box::new(sub_matches)));
                return Ok(matches);
            }

            if !only_positionals && let Some(long_name) = token.strip_prefix("--") {
                let (name, inline_val) = match long_name.split_once('=') {
                    Some((k, v)) => (k, Some(v.to_string())),
                    None => (long_name, None),
                };

                let matched_arg = effective_args
                    .iter()
                    .find(|a| a.long == Some(name))
                    .ok_or_else(|| {
                        XarpError::Parse(
                            self.format_error(&format!("unexpected argument '--{name}' found")),
                        )
                    })?;

                // Flags never take `=value`. Reject `--flag=value` for `SetTrue`
                // instead of silently ignoring the value.
                if matched_arg.action == ArgAction::SetTrue && inline_val.is_some() {
                    return Err(XarpError::Parse(self.format_error(&format!(
                        "argument '--{name}' does not take a value"
                    ))));
                }
                // Built-in handlers only when injected (no user override).
                if matched_arg.id == "help" && help_injected {
                    return Err(XarpError::Help(self.render_help()));
                }
                if matched_arg.id == "version" && version_injected {
                    return Err(XarpError::Version(self.render_version()));
                }

                match matched_arg.action {
                    ArgAction::SetTrue => {
                        matches.flags.insert(matched_arg.id.to_string());
                        explicit.insert(matched_arg.id.to_string());
                    }
                    ArgAction::Set => {
                        let value = if let Some(val) = inline_val {
                            val
                        } else {
                            i += 1;
                            if i >= tokens.len() || tokens[i] == "--" {
                                return Err(XarpError::Parse(self.format_error(&format!(
                                    "argument '--{name}' requires a value"
                                ))));
                            }
                            tokens[i].clone()
                        };
                        // Single-value options: last occurrence wins.
                        matches
                            .values
                            .insert(matched_arg.id.to_string(), vec![value]);
                        explicit.insert(matched_arg.id.to_string());
                    }
                    ArgAction::Append => {
                        let value = if let Some(val) = inline_val {
                            val
                        } else {
                            i += 1;
                            if i >= tokens.len() || tokens[i] == "--" {
                                return Err(XarpError::Parse(self.format_error(&format!(
                                    "argument '--{name}' requires a value"
                                ))));
                            }
                            tokens[i].clone()
                        };
                        matches
                            .values
                            .entry(matched_arg.id.to_string())
                            .or_default()
                            .push(value);
                        explicit.insert(matched_arg.id.to_string());
                    }
                }
            } else if !only_positionals && token.starts_with('-') && token.len() > 1 {
                let chars: Vec<char> = token[1..].chars().collect();
                let mut c_idx = 0;

                while c_idx < chars.len() {
                    let short_char = chars[c_idx];
                    let matched_arg = effective_args
                        .iter()
                        .find(|a| a.short == Some(short_char))
                        .ok_or_else(|| {
                            XarpError::Parse(self.format_error(&format!(
                                "unexpected argument '-{short_char}' found"
                            )))
                        })?;

                    match matched_arg.action {
                        ArgAction::SetTrue => {
                            // Built-in handlers work bundled too (`-vh` shows help).
                            // User overrides (no injection) behave as ordinary flags.
                            if matched_arg.id == "help" && help_injected {
                                return Err(XarpError::Help(self.render_help()));
                            }
                            if matched_arg.id == "version" && version_injected {
                                return Err(XarpError::Version(self.render_version()));
                            }
                            matches.flags.insert(matched_arg.id.to_string());
                            explicit.insert(matched_arg.id.to_string());
                            c_idx += 1;
                        }
                        ArgAction::Set | ArgAction::Append => {
                            // Support attached values (e.g., -p8080) and separated values (e.g., -p 8080).
                            // A leading '=' is stripped so `-p=8080` behaves like `--port=8080`.
                            let raw_value = if c_idx + 1 < chars.len() {
                                chars[c_idx + 1..].iter().collect::<String>()
                            } else {
                                i += 1;
                                if i >= tokens.len() || tokens[i] == "--" {
                                    return Err(XarpError::Parse(self.format_error(&format!(
                                        "argument '-{short_char}' requires a value"
                                    ))));
                                }
                                tokens[i].clone()
                            };
                            let value = raw_value
                                .strip_prefix('=')
                                .unwrap_or(&raw_value)
                                .to_string();

                            if matched_arg.action == ArgAction::Set {
                                matches
                                    .values
                                    .insert(matched_arg.id.to_string(), vec![value]);
                            } else {
                                matches
                                    .values
                                    .entry(matched_arg.id.to_string())
                                    .or_default()
                                    .push(value);
                            }
                            explicit.insert(matched_arg.id.to_string());
                            break;
                        }
                    }
                }
            } else {
                // Positional Argument: behavior depends on the declared action.
                // `Set` consumes one slot, `Append` (only allowed last)
                // collects all remaining positionals, `SetTrue` records
                // presence as a flag.
                if let Some(pos_arg) = positional_args.get(positional_idx) {
                    let pos_id = pos_arg.id.to_string();
                    let pos_action = pos_arg.action;
                    match pos_action {
                        ArgAction::SetTrue => {
                            matches.flags.insert(pos_id.clone());
                            explicit.insert(pos_id);
                            positional_idx += 1;
                        }
                        ArgAction::Set => {
                            matches
                                .values
                                .entry(pos_id.clone())
                                .or_default()
                                .push(token.clone());
                            explicit.insert(pos_id);
                            positional_idx += 1;
                        }
                        ArgAction::Append => {
                            matches
                                .values
                                .entry(pos_id.clone())
                                .or_default()
                                .push(token.clone());
                            explicit.insert(pos_id);
                            // Stay on the same index to collect further values.
                        }
                    }
                } else {
                    return Err(XarpError::Parse(
                        self.format_error(&format!("unexpected argument '{token}' found")),
                    ));
                }
            }

            i += 1;
        }

        // Apply environment variables, defaults, and required checks,
        // then validate possible values (all sources) and conflicts
        // (explicit selections only).
        self.apply_env_defaults(
            &effective_args,
            &mut matches,
            &mut explicit,
            env_lookup,
            false,
        )?;
        self.validate_values(&effective_args, &matches, &explicit)?;

        Ok(matches)
    }

    /// Returns the formatted application version string.
    #[must_use]
    pub fn render_version(&self) -> String {
        let v = self.version.unwrap_or("unknown");
        format!(
            "{} {}",
            self.styles.literal.paint(self.name),
            self.styles.placeholder.paint(v)
        )
    }

    /// Prints the application name and version string to standard output.
    pub fn print_version(&self) {
        println!("{}", self.render_version());
    }

    /// Renders the complete formatted help guide as a string.
    ///
    /// All coloring comes from `self.styles`, so `Styles::plain()` (or
    /// `NO_COLOR`) yields plain output and custom themes are honored.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn render_help(&self) -> String {
        let mut out = String::new();
        // Theme roles: header for structure, literal for commands/flags,
        // placeholder for values, usage for usage syntax, muted for secondary
        // notes, warning for required markers, valid for tips.
        let bar_accent = self.styles.header;
        let badge_name = self.styles.literal;
        let ver_pill = self.styles.placeholder;
        let about_txt = self.styles.muted;

        let sec_arrow = self.styles.header;
        let sec_title = self.styles.header;

        let prompt_glyph = self.styles.muted;
        let app_cmd = self.styles.literal;
        let subcmd_syntax = self.styles.usage;
        let pos_syntax = self.styles.placeholder;
        let opt_syntax = self.styles.usage;

        let cmd_item = self.styles.literal;
        let arg_item = self.styles.placeholder;
        let opt_flag = self.styles.literal;
        let opt_val = self.styles.placeholder;
        // Descriptions stay unstyled (as in Clap) so they remain readable
        // under any theme and are plain when `Styles::plain()` is used.
        let item_help = Style::new();

        let req_tag = self.styles.warning;
        let opt_tag = self.styles.muted;
        let def_tag = self.styles.muted;

        let tip_badge = self.styles.valid;
        let tip_text = self.styles.muted;

        let _ = writeln!(out);

        let v_str = self.version.unwrap_or("unknown");
        let name_upper = self.name.to_uppercase();

        let _ = write!(
            out,
            "{} {}  {} ",
            bar_accent.paint("┃"),
            badge_name.paint(&format!(" {name_upper} ")),
            ver_pill.paint(&format!("v{v_str}"))
        );

        if let Some(about) = self.about {
            let _ = write!(out, " {}", about_txt.paint(about));
        }
        let _ = writeln!(out, "\n{}", bar_accent.paint("┃"));

        let _ = write!(
            out,
            "{} {} {}",
            bar_accent.paint("┃"),
            prompt_glyph.paint("$"),
            app_cmd.paint(self.name)
        );

        if self.args.iter().any(|a| !a.is_positional()) {
            let _ = write!(out, " {}", opt_syntax.paint("[OPTIONS]"));
        }
        for pos in self.args.iter().filter(|a| a.is_positional()) {
            let val = pos.value_name.unwrap_or(pos.id);
            if pos.required {
                let _ = write!(out, " {}", pos_syntax.paint(&format!("<{val}>")));
            } else {
                let _ = write!(out, " {}", opt_syntax.paint(&format!("[{val}]")));
            }
        }
        if !self.subcommands.is_empty() {
            let _ = write!(out, " {}", subcmd_syntax.paint("[COMMAND]"));
        }
        let _ = writeln!(out, "\n");

        if !self.subcommands.is_empty() {
            let _ = writeln!(
                out,
                "{} {}",
                sec_arrow.paint("›"),
                sec_title.paint("Commands")
            );
            for cmd in &self.subcommands {
                let about = cmd.about.unwrap_or("");
                // Pad plain text before painting so ANSI codes don't break alignment.
                let name_padded = format!("{: <16}", cmd.name);
                let _ = writeln!(out, "  {}  {about}", cmd_item.paint(&name_padded),);
            }
            let _ = writeln!(out);
        }

        let positionals: Vec<&Arg> = self.args.iter().filter(|a| a.is_positional()).collect();
        if !positionals.is_empty() {
            let _ = writeln!(
                out,
                "{} {}",
                sec_arrow.paint("›"),
                sec_title.paint("Arguments")
            );
            for arg in positionals {
                let name = arg.value_name.unwrap_or(arg.id);
                let help = arg.help.unwrap_or("");
                let mut notes = if arg.required {
                    format!("{}", req_tag.paint("[required]"))
                } else {
                    format!("{}", opt_tag.paint("[optional]"))
                };
                if let Some(def) = arg.default_value {
                    let _ = write!(
                        notes,
                        " {}",
                        def_tag.paint(&format!("(default: \"{def}\")"))
                    );
                }
                if !arg.possible_values.is_empty() {
                    let allowed = arg.possible_values.join(", ");
                    let _ = write!(
                        notes,
                        " {}",
                        def_tag.paint(&format!("[possible values: {allowed}]"))
                    );
                }
                if let Some(env_key) = arg.env {
                    let _ = write!(notes, " {}", def_tag.paint(&format!("[env: {env_key}]")));
                }
                if arg.conflicts_with.len() == 1 {
                    let _ = write!(
                        notes,
                        " {}",
                        def_tag.paint(&format!("[conflicts: {}]", arg.conflicts_with[0]))
                    );
                } else if !arg.conflicts_with.is_empty() {
                    let list = arg.conflicts_with.join(", ");
                    let _ = write!(notes, " {}", def_tag.paint(&format!("[conflicts: {list}]")));
                }

                // Pad plain values before painting for correct columns.
                let name_padded = format!("{: <16}", format!("<{name}>"));
                let help_padded = format!("{help:<44}");
                let _ = writeln!(
                    out,
                    "  {}  {} {}",
                    arg_item.paint(&name_padded),
                    item_help.paint(&help_padded),
                    notes
                );
            }
            let _ = writeln!(out);
        }

        let options: Vec<&Arg> = self.args.iter().filter(|a| !a.is_positional()).collect();
        let _ = writeln!(
            out,
            "{} {}",
            sec_arrow.paint("›"),
            sec_title.paint("Flags & Options")
        );

        for arg in &options {
            let mut syntax = String::new();
            if let Some(s) = arg.short {
                let _ = write!(syntax, "-{s}, ");
            } else {
                syntax.push_str("    ");
            }
            if let Some(l) = arg.long {
                let _ = write!(syntax, "--{l}");
            }
            let val_plain = if arg.action == ArgAction::SetTrue {
                String::new()
            } else {
                format!(" <{}>", arg.value_name.unwrap_or("VAL"))
            };
            // Align on visible width (plain lengths), then paint parts.
            let visible_len = syntax.len() + val_plain.len();
            let pad = " ".repeat(24usize.saturating_sub(visible_len));
            let rendered = format!(
                "{}{val_painted}{pad}",
                opt_flag.paint(&syntax),
                val_painted = if val_plain.is_empty() {
                    String::new()
                } else {
                    format!("{}", opt_val.paint(&val_plain))
                }
            );

            let help = arg.help.unwrap_or("");
            let help_padded = format!("{help:<40}");
            let mut notes = String::new();
            if arg.required {
                let _ = write!(notes, "{}", req_tag.paint("[required]"));
            }
            if let Some(def) = arg.default_value {
                if !notes.is_empty() {
                    notes.push(' ');
                }
                let _ = write!(notes, "{}", def_tag.paint(&format!("(default: \"{def}\")")));
            }
            if !arg.possible_values.is_empty() {
                if !notes.is_empty() {
                    notes.push(' ');
                }
                let allowed = arg.possible_values.join(", ");
                let _ = write!(
                    notes,
                    "{}",
                    def_tag.paint(&format!("[possible values: {allowed}]"))
                );
            }
            if let Some(env_key) = arg.env {
                if !notes.is_empty() {
                    notes.push(' ');
                }
                let _ = write!(notes, "{}", def_tag.paint(&format!("[env: {env_key}]")));
            }
            if arg.conflicts_with.len() == 1 {
                if !notes.is_empty() {
                    notes.push(' ');
                }
                let _ = write!(
                    notes,
                    "{}",
                    def_tag.paint(&format!("[conflicts: {}]", arg.conflicts_with[0]))
                );
            } else if !arg.conflicts_with.is_empty() {
                if !notes.is_empty() {
                    notes.push(' ');
                }
                let list = arg.conflicts_with.join(", ");
                let _ = write!(notes, "{}", def_tag.paint(&format!("[conflicts: {list}]")));
            }

            let _ = writeln!(
                out,
                "  {}  {} {}",
                rendered,
                item_help.paint(&help_padded),
                notes
            );
        }

        if !options
            .iter()
            .any(|a| a.short == Some('h') || a.long == Some("help"))
        {
            let syntax = "  -h, --help";
            let pad = " ".repeat(24usize.saturating_sub(syntax.len()));
            let _ = writeln!(
                out,
                "  {}{}  {} ",
                opt_flag.paint(&syntax),
                pad,
                item_help.paint(&format!("{:<40}", "Show this help guide")),
            );
        }
        if self.version.is_some()
            && !options
                .iter()
                .any(|a| a.short == Some('V') || a.long == Some("version"))
        {
            let syntax = "  -V, --version";
            let pad = " ".repeat(24usize.saturating_sub(syntax.len()));
            let _ = writeln!(
                out,
                "  {}{}  {} ",
                opt_flag.paint(&syntax),
                pad,
                item_help.paint(&format!("{:<40}", "Show version number")),
            );
        }

        if !self.subcommands.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "  {} {} {}{}",
                tip_badge.paint(" TIP "),
                tip_text.paint("Run"),
                opt_flag.paint(&format!("{} <command> --help", self.name)),
                tip_text.paint(" for details on a specific subcommand.")
            );
        }
        let _ = writeln!(out);

        out
    }

    /// Prints formatted command-line help information to standard output.
    pub fn print_help(&self) {
        print!("{}", self.render_help());
    }

    fn format_error(&self, msg: &str) -> String {
        format!(
            "{}: {}\n\nFor more information, try '{}'.",
            self.styles.error.paint("error"),
            msg,
            self.styles.literal.paint("--help")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Styles;

    fn argv(args: &[&str]) -> Vec<String> {
        args.iter().map(ToString::to_string).collect()
    }

    fn empty_env() -> HashMap<String, String> {
        HashMap::new()
    }

    #[test]
    fn defaults_do_not_conflict_but_explicit_selections_do() {
        let app = || {
            Xarp::new("t")
                .arg(
                    Arg::new("a")
                        .long("aaa")
                        .default_value("x")
                        .conflicts_with("b"),
                )
                .arg(Arg::new("b").long("bbb").default_value("y"))
        };
        assert!(app().try_get_matches_from(&argv(&["t"])).is_ok());
        assert!(
            app()
                .try_get_matches_from(&argv(&["t", "--aaa", "1", "--bbb", "2"]))
                .is_err()
        );
    }

    #[test]
    fn unknown_conflict_target_is_rejected() {
        let result = Xarp::new("t")
            .arg(Arg::new("a").long("aaa").conflicts_with("typo"))
            .try_get_matches_from(&argv(&["t"]));
        assert!(matches!(result, Err(XarpError::Parse(_))));
    }

    #[test]
    fn illegal_definitions_are_rejected() {
        assert!(
            Xarp::new("t")
                .arg(Arg::new("").long("x"))
                .try_get_matches_from(&argv(&["t"]))
                .is_err()
        );
        assert!(
            Xarp::new("t")
                .arg(Arg::new("a").short('-'))
                .try_get_matches_from(&argv(&["t"]))
                .is_err()
        );
        assert!(
            Xarp::new("t")
                .arg(Arg::new("a").long("has space"))
                .try_get_matches_from(&argv(&["t"]))
                .is_err()
        );
        assert!(Xarp::new("").try_get_matches_from(&argv(&["t"])).is_err());
        assert!(
            Xarp::new("t")
                .subcommand(Xarp::new("-bad"))
                .try_get_matches_from(&argv(&["t"]))
                .is_err()
        );
    }

    #[test]
    fn duplicate_definitions_are_rejected() {
        assert!(
            Xarp::new("t")
                .arg(Arg::new("a").short('x'))
                .arg(Arg::new("b").short('x'))
                .try_get_matches_from(&argv(&["t"]))
                .is_err()
        );
        assert!(
            Xarp::new("t")
                .arg(Arg::new("a").long("dup"))
                .arg(Arg::new("b").long("dup"))
                .try_get_matches_from(&argv(&["t"]))
                .is_err()
        );
        assert!(
            Xarp::new("t")
                .arg(Arg::new("same"))
                .arg(Arg::new("same"))
                .try_get_matches_from(&argv(&["t"]))
                .is_err()
        );
    }

    #[test]
    fn required_positional_must_precede_optional_ones() {
        assert!(
            Xarp::new("t")
                .arg(Arg::new("a"))
                .arg(Arg::new("b").required(true))
                .try_get_matches_from(&argv(&["t"]))
                .is_err()
        );
    }

    #[test]
    fn single_value_options_keep_the_last_occurrence() {
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt"))
            .try_get_matches_from(&argv(&["t", "--opt", "a", "--opt", "b"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some("b".to_string()));
    }

    #[test]
    fn flag_defaults_and_env_precedence() {
        let app = || {
            Xarp::new("t").arg(
                Arg::new("f")
                    .long("flag")
                    .action(ArgAction::SetTrue)
                    .env("XARP_TEST_FLAG")
                    .default_value("true"),
            )
        };
        // Default applies when the environment is silent.
        let matches = app()
            .try_get_matches_with_env(&argv(&["t"]), &empty_env())
            .unwrap();
        assert!(matches.get_flag("f"));
        // An explicitly falsy environment value overrides a truthy default.
        let env_map = HashMap::from([("XARP_TEST_FLAG".to_string(), "0".to_string())]);
        let matches = app()
            .try_get_matches_with_env(&argv(&["t"]), &env_map)
            .unwrap();
        assert!(!matches.get_flag("f"));
        // A required flag is not satisfied by a falsy environment value.
        let result = Xarp::new("t")
            .arg(
                Arg::new("f")
                    .long("flag")
                    .action(ArgAction::SetTrue)
                    .env("XARP_TEST_FLAG")
                    .required(true),
            )
            .try_get_matches_with_env(&argv(&["t"]), &env_map);
        assert!(matches!(result, Err(XarpError::Parse(_))));
    }

    #[test]
    fn positional_actions_are_respected() {
        let matches = Xarp::new("t")
            .arg(Arg::new("p").action(ArgAction::SetTrue))
            .try_get_matches_from(&argv(&["t", "hello"]))
            .unwrap();
        assert!(matches.get_flag("p"));

        let matches = Xarp::new("t")
            .arg(Arg::new("files").action(ArgAction::Append))
            .try_get_matches_from(&argv(&["t", "a", "b", "c"]))
            .unwrap();
        assert_eq!(
            matches.get_many::<String>("files"),
            Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
    }

    #[test]
    fn subcommand_mode_keeps_parent_defaults() {
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("out").default_value("dflt"))
            .subcommand(Xarp::new("sub"))
            .try_get_matches_from(&argv(&["t", "sub"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some("dflt".to_string()));
    }

    #[test]
    fn delimiter_forces_positional_over_subcommand() {
        let matches = Xarp::new("t")
            .arg(Arg::new("input").required(true))
            .subcommand(Xarp::new("build"))
            .try_get_matches_from(&argv(&["t", "--", "build"]))
            .unwrap();
        assert_eq!(
            matches.get_one::<String>("input"),
            Some("build".to_string())
        );
        assert!(matches.subcommand().is_none());
    }

    #[test]
    fn delimiter_is_never_consumed_as_a_value() {
        let result = Xarp::new("t")
            .arg(Arg::new("o").long("opt"))
            .try_get_matches_from(&argv(&["t", "--opt", "--"]));
        assert!(matches!(result, Err(XarpError::Parse(_))));
    }

    #[test]
    fn short_attached_equals_is_stripped() {
        let matches = Xarp::new("t")
            .arg(Arg::new("p").short('p'))
            .try_get_matches_from(&argv(&["t", "-p=8080"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("p"), Some("8080".to_string()));
    }

    #[test]
    fn flag_with_inline_value_is_rejected() {
        let result = Xarp::new("t")
            .arg(Arg::new("f").long("flag").action(ArgAction::SetTrue))
            .try_get_matches_from(&argv(&["t", "--flag=false"]));
        assert!(matches!(result, Err(XarpError::Parse(_))));
    }

    #[test]
    fn bundled_help_and_version_flags_trigger() {
        let result = Xarp::new("t")
            .version("1.0")
            .arg(Arg::new("v").short('v').action(ArgAction::SetTrue))
            .try_get_matches_from(&argv(&["t", "-vh"]));
        assert!(matches!(result, Err(XarpError::Help(_))));

        let result = Xarp::new("t")
            .version("1.0")
            .arg(Arg::new("v").short('v').action(ArgAction::SetTrue))
            .try_get_matches_from(&argv(&["t", "-vV"]));
        assert!(matches!(result, Err(XarpError::Version(_))));
    }

    #[test]
    fn version_flag_without_version_is_unexpected() {
        assert!(
            Xarp::new("t")
                .try_get_matches_from(&argv(&["t", "--version"]))
                .is_err()
        );
    }

    #[test]
    fn user_defined_help_short_is_not_hijacked() {
        let matches = Xarp::new("t")
            .arg(Arg::new("myh").short('h').action(ArgAction::SetTrue))
            .try_get_matches_from(&argv(&["t", "-h"]))
            .unwrap();
        assert!(matches.get_flag("myh"));
    }

    #[test]
    fn typed_getters_distinguish_missing_from_invalid() {
        let matches = Xarp::new("t")
            .arg(Arg::new("port").long("port"))
            .try_get_matches_from(&argv(&["t", "--port", "abc"]))
            .unwrap();
        assert_eq!(matches.get_one::<u16>("port"), None);
        let error = matches.try_get_one::<u16>("port").unwrap_err();
        assert!(error.is_parse());
        assert!(error.to_string().contains("--help"));

        let missing = Xarp::new("t")
            .arg(Arg::new("port").long("port"))
            .try_get_matches_from(&argv(&["t"]))
            .unwrap();
        assert!(missing.try_get_one::<u16>("port").unwrap().is_none());
        assert!(missing.try_get_many::<u16>("port").unwrap().is_none());
    }

    #[test]
    fn error_kind_helpers_match_variants() {
        assert!(XarpError::Help(String::new()).is_help());
        assert!(XarpError::Version(String::new()).is_version());
        assert!(XarpError::Parse(String::new()).is_parse());
        assert!(!XarpError::Parse(String::new()).is_help());
    }

    #[test]
    fn with_env_is_deterministic() {
        let app = || Xarp::new("t").arg(Arg::new("o").long("out").env("OUT_ENV"));
        let env_map = HashMap::from([("OUT_ENV".to_string(), "mapped".to_string())]);
        let matches = app()
            .try_get_matches_with_env(&argv(&["t"]), &env_map)
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some("mapped".to_string()));
        assert!(
            app()
                .try_get_matches_with_env(&argv(&["t"]), &empty_env())
                .unwrap()
                .get_one::<String>("o")
                .is_none()
        );
    }

    #[test]
    fn plain_theme_help_contains_no_escapes() {
        let help = Xarp::new("t")
            .about("x")
            .styles(Styles::plain())
            .render_help();
        assert!(!help.contains('\x1b'));
    }

    #[test]
    fn help_shows_metadata_and_unknown_fallback() {
        let help = Xarp::new("t")
            .arg(
                Arg::new("o")
                    .short('o')
                    .long("out")
                    .required(true)
                    .value_name("F")
                    .possible_values(["a", "b"])
                    .env("MY_ENV")
                    .default_value("a"),
            )
            .render_help();
        for token in ["[required]", "possible values", "MY_ENV", "default"] {
            assert!(help.contains(token), "help is missing {token}");
        }
        let no_version = Xarp::new("t").about("x").render_help();
        assert!(no_version.contains("vunknown"));
    }

    #[test]
    fn long_space_separated_value_is_captured() {
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt"))
            .try_get_matches_from(&argv(&["t", "--opt", "val"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some("val".to_string()));
    }

    #[test]
    fn long_equals_value_is_captured() {
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt"))
            .try_get_matches_from(&argv(&["t", "--opt=val"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some("val".to_string()));
    }

    #[test]
    fn long_equals_empty_value_is_empty_string() {
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt"))
            .try_get_matches_from(&argv(&["t", "--opt="]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some(String::new()));
    }

    #[test]
    fn long_missing_value_at_end_errors() {
        let result = Xarp::new("t")
            .arg(Arg::new("o").long("opt"))
            .try_get_matches_from(&argv(&["t", "--opt"]));
        assert!(matches!(result, Err(XarpError::Parse(_))));
    }

    #[test]
    fn long_unknown_flag_errors() {
        let result = Xarp::new("t").try_get_matches_from(&argv(&["t", "--nope"]));
        let error = result.unwrap_err();
        assert!(error.is_parse());
        assert!(error.to_string().contains("--nope"));
    }

    #[test]
    fn long_value_may_start_with_a_dash() {
        let matches = Xarp::new("t")
            .arg(Arg::new("n").long("num"))
            .try_get_matches_from(&argv(&["t", "--num", "-1"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("n"), Some("-1".to_string()));
    }

    #[test]
    fn long_append_collects_every_occurrence() {
        let matches = Xarp::new("t")
            .arg(Arg::new("h").long("header").action(ArgAction::Append))
            .try_get_matches_from(&argv(&["t", "--header", "a", "--header=b"]))
            .unwrap();
        assert_eq!(
            matches.get_many::<String>("h"),
            Some(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn long_set_overwrites_across_forms() {
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt"))
            .try_get_matches_from(&argv(&["t", "--opt", "a", "--opt=b"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some("b".to_string()));
        assert_eq!(matches.get_many::<String>("o"), Some(vec!["b".to_string()]));
    }

    #[test]
    fn possible_values_accept_and_reject() {
        let ok = Xarp::new("t")
            .arg(Arg::new("m").long("mode").possible_values(["fast", "safe"]))
            .try_get_matches_from(&argv(&["t", "--mode", "safe"]))
            .unwrap();
        assert_eq!(ok.get_one::<String>("m"), Some("safe".to_string()));
        let bad = Xarp::new("t")
            .arg(Arg::new("m").long("mode").possible_values(["fast", "safe"]))
            .try_get_matches_from(&argv(&["t", "--mode", "wild"]));
        assert!(matches!(bad, Err(XarpError::Parse(_))));
    }

    #[test]
    fn invalid_default_against_possible_values_errors() {
        let result = Xarp::new("t")
            .arg(
                Arg::new("m")
                    .long("mode")
                    .possible_values(["fast", "safe"])
                    .default_value("wild"),
            )
            .try_get_matches_from(&argv(&["t"]));
        assert!(matches!(result, Err(XarpError::Parse(_))));
    }

    #[test]
    fn short_separate_and_attached_values() {
        let separate = Xarp::new("t")
            .arg(Arg::new("p").short('p'))
            .try_get_matches_from(&argv(&["t", "-p", "8080"]))
            .unwrap();
        assert_eq!(separate.get_one::<String>("p"), Some("8080".to_string()));
        let attached = Xarp::new("t")
            .arg(Arg::new("p").short('p'))
            .try_get_matches_from(&argv(&["t", "-p8080"]))
            .unwrap();
        assert_eq!(attached.get_one::<String>("p"), Some("8080".to_string()));
    }

    #[test]
    fn short_attached_equals_edge_cases() {
        let empty = Xarp::new("t")
            .arg(Arg::new("p").short('p'))
            .try_get_matches_from(&argv(&["t", "-p="]))
            .unwrap();
        assert_eq!(empty.get_one::<String>("p"), Some(String::new()));
        let doubled = Xarp::new("t")
            .arg(Arg::new("p").short('p'))
            .try_get_matches_from(&argv(&["t", "-p==x"]))
            .unwrap();
        assert_eq!(doubled.get_one::<String>("p"), Some("=x".to_string()));
    }

    #[test]
    fn short_missing_and_unknown_flag_errors() {
        assert!(
            Xarp::new("t")
                .arg(Arg::new("p").short('p'))
                .try_get_matches_from(&argv(&["t", "-p"]))
                .is_err()
        );
        let unknown = Xarp::new("t").try_get_matches_from(&argv(&["t", "-z"]));
        let error = unknown.unwrap_err();
        assert!(error.is_parse());
        assert!(error.to_string().contains("-z"));
    }

    #[test]
    fn short_flags_bundle_together() {
        let matches = Xarp::new("t")
            .arg(Arg::new("a").short('a').action(ArgAction::SetTrue))
            .arg(Arg::new("b").short('b').action(ArgAction::SetTrue))
            .try_get_matches_from(&argv(&["t", "-ab"]))
            .unwrap();
        assert!(matches.get_flag("a"));
        assert!(matches.get_flag("b"));
    }

    #[test]
    fn short_bundle_with_trailing_value() {
        let matches = Xarp::new("t")
            .arg(Arg::new("v").short('v').action(ArgAction::SetTrue))
            .arg(Arg::new("p").short('p'))
            .try_get_matches_from(&argv(&["t", "-vp8080"]))
            .unwrap();
        assert!(matches.get_flag("v"));
        assert_eq!(matches.get_one::<String>("p"), Some("8080".to_string()));
    }

    #[test]
    fn short_set_last_wins_and_append_collects() {
        let set = Xarp::new("t")
            .arg(Arg::new("p").short('p'))
            .try_get_matches_from(&argv(&["t", "-p", "a", "-p", "b"]))
            .unwrap();
        assert_eq!(set.get_one::<String>("p"), Some("b".to_string()));
        let append = Xarp::new("t")
            .arg(Arg::new("h").short('H').action(ArgAction::Append))
            .try_get_matches_from(&argv(&["t", "-H", "a", "-Hb"]))
            .unwrap();
        assert_eq!(
            append.get_many::<String>("h"),
            Some(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn short_value_may_start_with_a_dash() {
        let matches = Xarp::new("t")
            .arg(Arg::new("p").short('p'))
            .try_get_matches_from(&argv(&["t", "-p", "-1"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("p"), Some("-1".to_string()));
    }

    #[test]
    fn lone_dash_is_a_positional_value() {
        let matches = Xarp::new("t")
            .arg(Arg::new("input"))
            .try_get_matches_from(&argv(&["t", "-"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("input"), Some("-".to_string()));
    }

    #[test]
    fn lone_dash_without_positionals_errors() {
        assert!(
            Xarp::new("t")
                .try_get_matches_from(&argv(&["t", "-"]))
                .is_err()
        );
    }

    #[test]
    fn multiple_positionals_bind_in_order() {
        let matches = Xarp::new("t")
            .arg(Arg::new("src"))
            .arg(Arg::new("dst"))
            .try_get_matches_from(&argv(&["t", "a.txt", "b.txt"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("src"), Some("a.txt".to_string()));
        assert_eq!(matches.get_one::<String>("dst"), Some("b.txt".to_string()));
    }

    #[test]
    fn missing_required_positional_errors() {
        assert!(
            Xarp::new("t")
                .arg(Arg::new("input").required(true))
                .try_get_matches_from(&argv(&["t"]))
                .is_err()
        );
    }

    #[test]
    fn unexpected_extra_positional_errors() {
        assert!(
            Xarp::new("t")
                .arg(Arg::new("input"))
                .try_get_matches_from(&argv(&["t", "a", "b"]))
                .is_err()
        );
    }

    #[test]
    fn optional_positional_may_be_omitted() {
        let matches = Xarp::new("t")
            .arg(Arg::new("input"))
            .try_get_matches_from(&argv(&["t"]))
            .unwrap();
        assert!(matches.get_one::<String>("input").is_none());
    }

    #[test]
    fn default_fills_omitted_positional() {
        let matches = Xarp::new("t")
            .arg(Arg::new("input").default_value("stdin"))
            .try_get_matches_from(&argv(&["t"]))
            .unwrap();
        assert_eq!(
            matches.get_one::<String>("input"),
            Some("stdin".to_string())
        );
    }

    #[test]
    fn possible_values_apply_to_positionals() {
        assert!(
            Xarp::new("t")
                .arg(Arg::new("mode").possible_values(["x", "y"]))
                .try_get_matches_from(&argv(&["t", "z"]))
                .is_err()
        );
    }

    #[test]
    fn non_last_append_positional_is_rejected() {
        assert!(
            Xarp::new("t")
                .arg(Arg::new("files").action(ArgAction::Append))
                .arg(Arg::new("other"))
                .try_get_matches_from(&argv(&["t", "a"]))
                .is_err()
        );
    }

    #[test]
    fn required_append_positional_needs_one_value() {
        assert!(
            Xarp::new("t")
                .arg(Arg::new("files").action(ArgAction::Append).required(true))
                .try_get_matches_from(&argv(&["t"]))
                .is_err()
        );
        let matches = Xarp::new("t")
            .arg(Arg::new("files").action(ArgAction::Append).required(true))
            .try_get_matches_from(&argv(&["t", "only"]))
            .unwrap();
        assert_eq!(
            matches.get_many::<String>("files"),
            Some(vec!["only".to_string()])
        );
    }

    #[test]
    fn flags_default_to_absent() {
        let matches = Xarp::new("t")
            .arg(Arg::new("v").long("verbose").action(ArgAction::SetTrue))
            .try_get_matches_from(&argv(&["t"]))
            .unwrap();
        assert!(!matches.get_flag("v"));
    }

    #[test]
    fn flag_set_via_long_and_short() {
        for token in ["--verbose", "-v"] {
            let matches = Xarp::new("t")
                .arg(
                    Arg::new("v")
                        .short('v')
                        .long("verbose")
                        .action(ArgAction::SetTrue),
                )
                .try_get_matches_from(&argv(&["t", token]))
                .unwrap();
            assert!(matches.get_flag("v"), "token {token} should set the flag");
        }
    }

    #[test]
    fn falsy_flag_defaults_leave_the_flag_unset() {
        for default in ["false", "0", "no"] {
            let matches = Xarp::new("t")
                .arg(
                    Arg::new("f")
                        .long("flag")
                        .action(ArgAction::SetTrue)
                        .default_value(default),
                )
                .try_get_matches_with_env(&argv(&["t"]), &empty_env())
                .unwrap();
            assert!(!matches.get_flag("f"), "default {default} should not set");
        }
    }

    #[test]
    fn truthy_matching_is_case_insensitive() {
        for value in ["TRUE", "True", "tRuE"] {
            let env_map = HashMap::from([("K".to_string(), value.to_string())]);
            let matches = Xarp::new("t")
                .arg(
                    Arg::new("f")
                        .long("flag")
                        .action(ArgAction::SetTrue)
                        .env("K"),
                )
                .try_get_matches_with_env(&argv(&["t"]), &env_map)
                .unwrap();
            assert!(matches.get_flag("f"), "env {value} should set the flag");
        }
    }

    #[test]
    fn cli_value_beats_default() {
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt").default_value("dflt"))
            .try_get_matches_from(&argv(&["t", "--opt", "cli"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some("cli".to_string()));
    }

    #[test]
    fn cli_value_beats_env_value() {
        let env_map = HashMap::from([("K".to_string(), "env".to_string())]);
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt").env("K"))
            .try_get_matches_with_env(&argv(&["t", "--opt", "cli"]), &env_map)
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some("cli".to_string()));
    }

    #[test]
    fn env_value_beats_default() {
        let env_map = HashMap::from([("K".to_string(), "env".to_string())]);
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt").env("K").default_value("dflt"))
            .try_get_matches_with_env(&argv(&["t"]), &env_map)
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some("env".to_string()));
    }

    #[test]
    fn required_option_errors_and_env_satisfies() {
        assert!(
            Xarp::new("t")
                .arg(Arg::new("c").long("cfg").required(true))
                .try_get_matches_with_env(&argv(&["t"]), &empty_env())
                .is_err()
        );
        let env_map = HashMap::from([("K".to_string(), "v".to_string())]);
        let matches = Xarp::new("t")
            .arg(Arg::new("c").long("cfg").required(true).env("K"))
            .try_get_matches_with_env(&argv(&["t"]), &env_map)
            .unwrap();
        assert_eq!(matches.get_one::<String>("c"), Some("v".to_string()));
    }

    #[test]
    fn required_option_satisfied_by_default() {
        let matches = Xarp::new("t")
            .arg(Arg::new("c").long("cfg").required(true).default_value("d"))
            .try_get_matches_from(&argv(&["t"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("c"), Some("d".to_string()));
    }

    #[test]
    fn cli_append_replaces_default_and_env() {
        let env_map = HashMap::from([("K".to_string(), "env".to_string())]);
        let matches = Xarp::new("t")
            .arg(
                Arg::new("h")
                    .long("header")
                    .action(ArgAction::Append)
                    .env("K")
                    .default_value("dflt"),
            )
            .try_get_matches_with_env(&argv(&["t", "--header", "cli"]), &env_map)
            .unwrap();
        assert_eq!(
            matches.get_many::<String>("h"),
            Some(vec!["cli".to_string()])
        );
    }

    #[test]
    fn empty_env_string_is_a_value_for_options() {
        let env_map = HashMap::from([("K".to_string(), String::new())]);
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt").env("K"))
            .try_get_matches_with_env(&argv(&["t"]), &env_map)
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some(String::new()));
    }

    #[test]
    fn conflicting_pair_errors_in_both_orders() {
        let app = || {
            Xarp::new("t")
                .arg(Arg::new("a").long("aaa").conflicts_with("b"))
                .arg(Arg::new("b").long("bbb"))
        };
        assert!(
            app()
                .try_get_matches_from(&argv(&["t", "--aaa", "1", "--bbb", "2"]))
                .is_err()
        );
        assert!(
            app()
                .try_get_matches_from(&argv(&["t", "--bbb", "2", "--aaa", "1"]))
                .is_err()
        );
        assert!(
            app()
                .try_get_matches_from(&argv(&["t", "--aaa", "1"]))
                .is_ok()
        );
    }

    #[test]
    fn env_selections_participate_in_conflicts() {
        let env_map = HashMap::from([("KB".to_string(), "2".to_string())]);
        let result = Xarp::new("t")
            .arg(Arg::new("a").long("aaa").conflicts_with("b"))
            .arg(Arg::new("b").long("bbb").env("KB"))
            .try_get_matches_with_env(&argv(&["t", "--aaa", "1"]), &env_map);
        assert!(matches!(result, Err(XarpError::Parse(_))));
    }

    #[test]
    fn basic_subcommand_routing_with_flag() {
        let matches = Xarp::new("t")
            .subcommand(
                Xarp::new("run").arg(Arg::new("fast").long("fast").action(ArgAction::SetTrue)),
            )
            .try_get_matches_from(&argv(&["t", "run", "--fast"]))
            .unwrap();
        let (name, sub) = matches.subcommand().unwrap();
        assert_eq!(name, "run");
        assert!(sub.get_flag("fast"));
    }

    #[test]
    fn subcommand_required_args_are_enforced() {
        assert!(
            Xarp::new("t")
                .subcommand(Xarp::new("run").arg(Arg::new("target").required(true)))
                .try_get_matches_from(&argv(&["t", "run"]))
                .is_err()
        );
    }

    #[test]
    fn unknown_bare_token_without_subcommands_errors() {
        assert!(
            Xarp::new("t")
                .try_get_matches_from(&argv(&["t", "nope"]))
                .is_err()
        );
    }

    #[test]
    fn parent_option_before_subcommand_is_kept() {
        let matches = Xarp::new("t")
            .arg(Arg::new("v").long("verbose").action(ArgAction::SetTrue))
            .subcommand(Xarp::new("run"))
            .try_get_matches_from(&argv(&["t", "--verbose", "run"]))
            .unwrap();
        assert!(matches.get_flag("v"));
        assert_eq!(matches.subcommand().unwrap().0, "run");
    }

    #[test]
    fn subcommand_help_returns_help() {
        let result = Xarp::new("t")
            .subcommand(Xarp::new("run"))
            .try_get_matches_from(&argv(&["t", "run", "--help"]));
        assert!(matches!(result, Err(XarpError::Help(_))));
    }

    #[test]
    fn nested_subcommands_route_recursively() {
        let matches = Xarp::new("t")
            .subcommand(Xarp::new("outer").subcommand(
                Xarp::new("inner").arg(Arg::new("x").long("x").action(ArgAction::SetTrue)),
            ))
            .try_get_matches_from(&argv(&["t", "outer", "inner", "--x"]))
            .unwrap();
        let (outer, outer_matches) = matches.subcommand().unwrap();
        assert_eq!(outer, "outer");
        let (inner, inner_matches) = outer_matches.subcommand().unwrap();
        assert_eq!(inner, "inner");
        assert!(inner_matches.get_flag("x"));
    }

    #[test]
    fn delimiter_before_subcommand_name_errors_without_positionals() {
        assert!(
            Xarp::new("t")
                .subcommand(Xarp::new("run"))
                .try_get_matches_from(&argv(&["t", "--", "run"]))
                .is_err()
        );
    }

    #[test]
    fn exact_help_and_version_tokens() {
        assert!(
            Xarp::new("t")
                .try_get_matches_from(&argv(&["t", "-h"]))
                .unwrap_err()
                .is_help()
        );
        assert!(
            Xarp::new("t")
                .try_get_matches_from(&argv(&["t", "--help"]))
                .unwrap_err()
                .is_help()
        );
        assert!(
            Xarp::new("t")
                .version("9.9")
                .try_get_matches_from(&argv(&["t", "-V"]))
                .unwrap_err()
                .is_version()
        );
    }

    #[test]
    fn help_payload_contains_app_name() {
        let error = Xarp::new("demo")
            .try_get_matches_from(&argv(&["demo", "--help"]))
            .unwrap_err();
        assert!(error.to_string().contains("demo"));
    }

    #[test]
    fn version_payload_contains_name_and_version() {
        let error = Xarp::new("demo")
            .version("3.2.1")
            .try_get_matches_from(&argv(&["demo", "--version"]))
            .unwrap_err();
        let text = error.to_string();
        assert!(text.contains("demo"));
        assert!(text.contains("3.2.1"));
    }

    #[test]
    fn render_version_unknown_fallback() {
        assert!(Xarp::new("demo").render_version().contains("unknown"));
    }

    #[test]
    fn help_lists_sections_and_builtin_flags() {
        let help = Xarp::new("demo")
            .version("1.0")
            .about("d")
            .arg(Arg::new("input").required(true))
            .subcommand(Xarp::new("run").about("run it"))
            .render_help();
        for token in [
            "Commands",
            "Arguments",
            "Flags & Options",
            "-h, --help",
            "-V, --version",
        ] {
            assert!(help.contains(token), "help is missing {token}");
        }
    }

    #[test]
    fn custom_long_help_override_is_not_hijacked() {
        let matches = Xarp::new("t")
            .arg(Arg::new("myhelp").long("help").action(ArgAction::SetTrue))
            .try_get_matches_from(&argv(&["t", "--help"]))
            .unwrap();
        assert!(matches.get_flag("myhelp"));
    }

    #[test]
    fn custom_version_short_override_is_ordinary() {
        let matches = Xarp::new("t")
            .version("1.0")
            .arg(Arg::new("mine").short('V').action(ArgAction::SetTrue))
            .try_get_matches_from(&argv(&["t", "-V"]))
            .unwrap();
        assert!(matches.get_flag("mine"));
    }

    #[test]
    fn error_display_and_from_string() {
        let parse = XarpError::from("boom".to_string());
        assert!(parse.is_parse());
        assert_eq!(parse.to_string(), "boom");
        assert_eq!(XarpError::Help("h".to_string()).to_string(), "h");
        assert_eq!(XarpError::Version("v".to_string()).to_string(), "v");
    }

    #[test]
    fn from_arg_value_across_types() {
        use crate::FromArgValue;
        assert_eq!(u16::from_arg_value("8080"), Some(8080));
        assert_eq!(u16::from_arg_value("abc"), None);
        assert_eq!(i32::from_arg_value("-5"), Some(-5));
        assert_eq!(bool::from_arg_value("true"), Some(true));
        assert_eq!(bool::from_arg_value("yes"), None);
        assert_eq!(String::from_arg_value("hi"), Some("hi".to_string()));
    }

    #[test]
    fn getters_return_none_when_absent() {
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt"))
            .try_get_matches_from(&argv(&["t"]))
            .unwrap();
        assert!(!matches.get_flag("missing"));
        assert!(matches.get_one::<String>("o").is_none());
        assert!(matches.get_many::<String>("o").is_none());
        assert!(matches.subcommand().is_none());
    }

    #[test]
    fn arg_builder_defaults() {
        let arg = Arg::new("x");
        assert!(arg.is_positional());
        assert!(!arg.required);
        assert_eq!(arg.action, ArgAction::Set);
        assert!(arg.short.is_none());
        assert!(arg.long.is_none());
        let flagged = Arg::new("y").short('y').long("why");
        assert!(!flagged.is_positional());
    }

    #[test]
    fn empty_argv_applies_defaults() {
        let matches = Xarp::new("t")
            .arg(Arg::new("o").long("opt").default_value("d"))
            .try_get_matches_from(&[])
            .unwrap();
        assert_eq!(matches.get_one::<String>("o"), Some("d".to_string()));
    }

    #[test]
    fn bare_delimiter_with_no_positionals_is_ok() {
        assert!(
            Xarp::new("t")
                .try_get_matches_from(&argv(&["t", "--"]))
                .is_ok()
        );
    }

    #[test]
    fn append_positional_collects_after_delimiter() {
        let matches = Xarp::new("t")
            .arg(Arg::new("files").action(ArgAction::Append))
            .try_get_matches_from(&argv(&["t", "--", "-a", "--help"]))
            .unwrap();
        assert_eq!(
            matches.get_many::<String>("files"),
            Some(vec!["-a".to_string(), "--help".to_string()])
        );
    }

    #[test]
    fn possible_values_checked_per_append_item() {
        assert!(
            Xarp::new("t")
                .arg(
                    Arg::new("m")
                        .long("mode")
                        .action(ArgAction::Append)
                        .possible_values(["a", "b"])
                )
                .try_get_matches_from(&argv(&["t", "--mode", "a", "--mode", "z"]))
                .is_err()
        );
    }

    #[test]
    fn single_sided_conflict_allows_the_other_side() {
        let matches = Xarp::new("t")
            .arg(Arg::new("a").long("aaa").conflicts_with("b"))
            .arg(Arg::new("b").long("bbb"))
            .try_get_matches_from(&argv(&["t", "--bbb", "2"]))
            .unwrap();
        assert_eq!(matches.get_one::<String>("b"), Some("2".to_string()));
    }
}
