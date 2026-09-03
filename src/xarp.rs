use crate::{
    color::Color,
    style::{Style, Styles},
};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write;
use std::process;

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
        }
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
    #[must_use]
    pub fn get_one<T: FromArgValue>(&self, id: &str) -> Option<T> {
        self.values
            .get(id)
            .and_then(|vals| vals.first())
            .and_then(|val| T::from_arg_value(val))
    }

    /// Gets all values supplied for an argument, parsed into `Vec<T>`.
    #[must_use]
    pub fn get_many<T: FromArgValue>(&self, id: &str) -> Option<Vec<T>> {
        self.values
            .get(id)
            .map(|vals| vals.iter().filter_map(|v| T::from_arg_value(v)).collect())
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
            Err(err) => {
                eprintln!("{err}");
                process::exit(2);
            }
        }
    }

    /// Attempts to parse arguments from a string slice without exiting on error.
    ///
    /// # Errors
    ///
    /// Returns an error message if an unexpected argument is encountered, a required
    /// argument is missing, or an option is missing its value.
    #[allow(clippy::too_many_lines)]
    pub fn try_get_matches_from(self, args: &[String]) -> Result<ArgMatches, String> {
        let mut matches = ArgMatches::default();
        let tokens = if args.is_empty() { &[] } else { &args[1..] };

        // Inject default built-in help and version flags
        let mut effective_args = self.args.clone();
        if !effective_args.iter().any(|a| a.id == "help") {
            effective_args.push(
                Arg::new("help")
                    .short('h')
                    .long("help")
                    .help("Print help information")
                    .action(ArgAction::SetTrue),
            );
        }
        if self.version.is_some() && !effective_args.iter().any(|a| a.id == "version") {
            effective_args.push(
                Arg::new("version")
                    .short('V')
                    .long("version")
                    .help("Print version information")
                    .action(ArgAction::SetTrue),
            );
        }

        let mut positional_idx = 0;
        let positional_args: Vec<&Arg> = effective_args
            .iter()
            .filter(|a| a.is_positional())
            .collect();

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

            // Subcommands
            if !only_positionals
                && !token.starts_with('-')
                && positional_idx == 0
                && let Some(sub) = self.subcommands.iter().find(|s| s.name == token)
            {
                let sub_matches = sub.clone().try_get_matches_from(&tokens[i..])?;
                matches.subcommand = Some((sub.name.to_string(), Box::new(sub_matches)));
                return Ok(matches);
            }

            if !only_positionals && (token == "-h" || token == "--help") {
                return Err(XarpError::Help(self.render_help()));
            } else if !only_positionals && (token == "-V" || token == "--version") {
                return Err(XarpError::Version(self.render_version()));
            } else if !only_positionals && let Some(long_name) = token.strip_prefix("--") {
                let (name, inline_val) = match long_name.split_once('=') {
                    Some((k, v)) => (k, Some(v.to_string())),
                    None => (long_name, None),
                };

                let matched_arg = effective_args
                    .iter()
                    .find(|a| a.long == Some(name))
                    .ok_or_else(|| {
                        self.format_error(&format!("unexpected argument '--{name}' found"))
                    })?;

                match matched_arg.action {
                    ArgAction::SetTrue => {
                        matches.flags.insert(matched_arg.id.to_string());
                    }
                    ArgAction::Set | ArgAction::Append => {
                        let value = if let Some(val) = inline_val {
                            val
                        } else {
                            i += 1;
                            if i >= tokens.len() {
                                return Err(self.format_error(&format!(
                                    "argument '--{name}' requires a value"
                                )));
                            }
                            tokens[i].clone()
                        };
                        matches
                            .values
                            .entry(matched_arg.id.to_string())
                            .or_default()
                            .push(value);
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
                            matches.flags.insert(matched_arg.id.to_string());
                            c_idx += 1;
                        }
                        ArgAction::Set | ArgAction::Append => {
                            // Support attached values (e.g., -p8080) and separated values (e.g., -p 8080)
                            let value = if c_idx + 1 < chars.len() {
                                let attached: String = chars[c_idx + 1..].iter().collect();
                                c_idx = chars.len();
                                attached
                            } else {
                                i += 1;
                                if i >= tokens.len() {
                                    return Err(XarpError::Parse(self.format_error(&format!(
                                        "argument '-{short_char}' requires a value"
                                    ))));
                                }
                                tokens[i].clone()
                            };

                            matches
                                .values
                                .entry(matched_arg.id.to_string())
                                .or_default()
                                .push(value);
                            break;
                        }
                    }
                }
            } else {
                // Positional Argument
                if let Some(pos_arg) = positional_args.get(positional_idx) {
                    matches
                        .values
                        .entry(pos_arg.id.to_string())
                        .or_default()
                        .push(token.clone());
                    positional_idx += 1;
                } else {
                    return Err(self.format_error(&format!("unexpected argument '{token}' found")));
                }
            }

            i += 1;
        }

        // Apply defaults & verify required arguments
        for arg in &effective_args {
            if !matches.values.contains_key(arg.id) {
                if let Some(default) = arg.default_value {
                    matches
                        .values
                        .insert(arg.id.to_string(), vec![default.to_string()]);
                } else if arg.required {
                    return Err(self.format_error(&format!(
                        "the required argument '{}' was not provided",
                        arg.id
                    )));
                }
            }
        }

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
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn render_help(&self) -> String {
        let mut out = String::new();
        let bar_accent = Style::new().bold().fg(Color::BrightCyan);
        let badge_name = Style::new().bold().bg(Color::Teal).fg(Color::Black);
        let ver_pill = Style::new().italic().fg(Color::Gold);
        let about_txt = Style::new().fg(Color::Silver);

        let sec_arrow = Style::new().bold().fg(Color::Orange);
        let sec_title = Style::new().bold().underline().fg(Color::BrightWhite);

        let prompt_glyph = Style::new().bold().fg(Color::BrightMagenta);
        let app_cmd = Style::new().bold().fg(Color::White);
        let subcmd_syntax = Style::new().bold().fg(Color::Lime);
        let pos_syntax = Style::new().italic().fg(Color::Gold);
        let opt_syntax = Style::new().dim().fg(Color::BrightCyan);

        let cmd_item = Style::new().bold().fg(Color::Lime);
        let arg_item = Style::new().italic().fg(Color::Gold);
        let opt_flag = Style::new().bold().fg(Color::BrightCyan);
        let opt_val = Style::new().fg(Color::Teal);
        let item_help = Style::new().fg(Color::BrightWhite);

        let req_tag = Style::new().bold().fg(Color::Pink);
        let opt_tag = Style::new().dim().fg(Color::BrightBlack);
        let def_tag = Style::new().fg(Color::Teal).dim();

        let tip_badge = Style::new().bold().bg(Color::Indigo).fg(Color::BrightWhite);
        let tip_text = Style::new().dim().fg(Color::Silver);

        let _ = writeln!(out);

        let v_str = self.version.unwrap_or("0.1.1");
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
                let _ = writeln!(
                    out,
                    "  {: <16}  {}",
                    cmd_item.paint(cmd.name),
                    item_help.paint(about)
                );
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
                let status = if arg.required {
                    format!("{}", req_tag.paint("[required]"))
                } else {
                    format!("{}", opt_tag.paint("[optional]"))
                };

                let _ = writeln!(
                    out,
                    "  {: <16}  {: <44} {}",
                    arg_item.paint(&format!("<{name}>")),
                    item_help.paint(help),
                    status
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

            let rendered_flag = opt_flag.paint(&syntax);
            let val_suffix = if arg.action == ArgAction::SetTrue {
                String::new()
            } else {
                format!(
                    " {}",
                    opt_val.paint(&format!("<{}>", arg.value_name.unwrap_or("VAL")))
                )
            };

            let combined_flag = format!("{rendered_flag}{val_suffix}");
            let help = arg.help.unwrap_or("");
            let default_note = if let Some(def) = arg.default_value {
                format!("{}", def_tag.paint(&format!("(default: \"{def}\")")))
            } else {
                String::new()
            };

            let _ = writeln!(
                out,
                "  {: <24}  {: <40} {}",
                combined_flag,
                item_help.paint(help),
                default_note
            );
        }

        if !options
            .iter()
            .any(|a| a.short == Some('h') || a.long == Some("help"))
        {
            let _ = writeln!(
                out,
                "  {: <24}  {}",
                opt_flag.paint("  -h, --help"),
                item_help.paint("Show this help guide")
            );
        }
        if self.version.is_some()
            && !options
                .iter()
                .any(|a| a.short == Some('V') || a.long == Some("version"))
        {
            let _ = writeln!(
                out,
                "  {: <24}  {}",
                opt_flag.paint("  -V, --version"),
                item_help.paint("Show version number")
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
