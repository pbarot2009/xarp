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
    /// Values that fail to parse are silently skipped.
    /// Use [`ArgMatches::try_get_many`] to surface parse failures as errors.
    #[must_use]
    pub fn get_many<T: FromArgValue>(&self, id: &str) -> Option<Vec<T>> {
        self.values
            .get(id)
            .map(|vals| vals.iter().filter_map(|v| T::from_arg_value(v)).collect())
    }

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
    pub fn try_get_one<T: FromArgValue>(&self, id: &str) -> Result<Option<T>, XarpError> {
        match self.values.get(id).and_then(|vals| vals.first()) {
            None => Ok(None),
            Some(raw) => match T::from_arg_value(raw) {
                Some(parsed) => Ok(Some(parsed)),
                None => Err(XarpError::Parse(format!(
                    "invalid value '{raw}' for '{id}'"
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
                                "invalid value '{raw}' for '{id}'"
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
    #[must_use]
    pub fn get_matches(self) -> ArgMatches {
        let args: Vec<String> = env::args().collect();
        match self.try_get_matches_from(&args) {
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

    /// Returns `true` for truthy flag values (`"1"` or case-insensitive `"true"`).
    fn is_truthy(val: &str) -> bool {
        val == "1" || val.eq_ignore_ascii_case("true")
    }

    /// Validates argument definitions: duplicate ids, shorts, longs and
    /// duplicate subcommand names.
    fn validate_definitions(&self) -> Result<(), XarpError> {
        let mut ids = HashSet::new();
        for arg in &self.args {
            if !ids.insert(arg.id) {
                return Err(XarpError::Parse(self.format_error(&format!(
                    "duplicate argument id '{}' found",
                    arg.id
                ))));
            }
        }
        let mut shorts = HashSet::new();
        for arg in &self.args {
            if let Some(short) = arg.short {
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
                if !longs.insert(long) {
                    return Err(XarpError::Parse(
                        self.format_error(&format!("duplicate long flag '--{long}' found")),
                    ));
                }
            }
        }
        let mut sub_names = HashSet::new();
        for sub in &self.subcommands {
            if !sub_names.insert(sub.name) {
                return Err(XarpError::Parse(self.format_error(&format!(
                    "duplicate subcommand '{0}' found",
                    sub.name
                ))));
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
        skip_required: bool,
    ) -> Result<(), XarpError> {
        for arg in effective_args {
            if matches.values.contains_key(arg.id) || matches.flags.contains(arg.id) {
                continue;
            }
            if let Some(env_key) = arg.env {
                if let Ok(env_val) = env::var(env_key) {
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
    /// # Errors
    ///
    /// Returns [`XarpError`] if parsing fails or when `--help`/`--version` flags are passed.
    #[allow(clippy::too_many_lines)]
    pub fn try_get_matches_from(self, args: &[String]) -> Result<ArgMatches, XarpError> {
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
                self.apply_env_defaults(&effective_args, &mut matches, &mut explicit, true)?;
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
                            if i >= tokens.len() {
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
                            if i >= tokens.len() {
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
                                if i >= tokens.len() {
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
        self.apply_env_defaults(&effective_args, &mut matches, &mut explicit, false)?;
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
