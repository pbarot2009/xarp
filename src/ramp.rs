use crate::{
    color::Color,
    effect::Effects,
    style::{Style, Styles},
};
use std::collections::{HashMap, HashSet};
use std::env;
use std::process;

// Argument Action & Definition
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgAction {
    /// Flag without value (e.g., `-v`, `--verbose`). Sets `true`.
    SetTrue,
    /// Option or Positional that captures a single value.
    Set,
    /// Option that can be supplied multiple times to collect values.
    Append,
}

#[derive(Clone, Debug)]
pub struct Arg {
    pub id: &'static str,
    pub short: Option<char>,
    pub long: Option<&'static str>,
    pub help: Option<&'static str>,
    pub value_name: Option<&'static str>,
    pub required: bool,
    pub action: ArgAction,
    pub default_value: Option<&'static str>,
}

impl Arg {
    /// Create a new argument definition with an identifier.
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

    pub fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    pub fn long(mut self, long: &'static str) -> Self {
        self.long = Some(long);
        self
    }

    pub fn help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }

    pub fn value_name(mut self, name: &'static str) -> Self {
        self.value_name = Some(name);
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn action(mut self, action: ArgAction) -> Self {
        self.action = action;
        self
    }

    pub fn default_value(mut self, default: &'static str) -> Self {
        self.default_value = Some(default);
        self
    }

    #[inline]
    pub fn is_positional(&self) -> bool {
        self.short.is_none() && self.long.is_none()
    }
}

// ArgMatches (Parsed Output Container)
#[derive(Clone, Debug, Default)]
pub struct ArgMatches {
    flags: HashSet<String>,
    values: HashMap<String, Vec<String>>,
    subcommand: Option<(String, Box<ArgMatches>)>,
}

impl ArgMatches {
    /// Checks if a boolean flag was supplied.
    pub fn get_flag(&self, id: &str) -> bool {
        self.flags.contains(id)
    }

    /// Gets the first or single value for an option/positional argument.
    pub fn get_one<T: FromArgValue>(&self, id: &str) -> Option<T> {
        self.values
            .get(id)
            .and_then(|vals| vals.first())
            .and_then(|val| T::from_arg_value(val))
    }

    /// Gets all values supplied for an argument (e.g. multiple occurrences).
    pub fn get_many<T: FromArgValue>(&self, id: &str) -> Option<Vec<T>> {
        self.values
            .get(id)
            .map(|vals| vals.iter().filter_map(|v| T::from_arg_value(v)).collect())
    }

    /// Returns the matched subcommand name and its inner matches.
    pub fn subcommand(&self) -> Option<(&str, &ArgMatches)> {
        self.subcommand
            .as_ref()
            .map(|(name, matches)| (name.as_str(), matches.as_ref()))
    }
}

/// Helper trait to parse strings into types like `String`, `i32`, `PathBuf`, etc.
pub trait FromArgValue: Sized {
    fn from_arg_value(val: &str) -> Option<Self>;
}

impl<T: std::str::FromStr> FromArgValue for T {
    fn from_arg_value(val: &str) -> Option<Self> {
        val.parse().ok()
    }
}

// ============================================================================
// 3. Ramp CLI Engine
// ============================================================================

#[derive(Clone, Debug)]
pub struct Ramp {
    pub name: &'static str,
    pub version: Option<&'static str>,
    pub about: Option<&'static str>,
    pub styles: Styles,
    pub args: Vec<Arg>,
    pub subcommands: Vec<Ramp>,
}

impl Ramp {
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

    pub fn version(mut self, version: &'static str) -> Self {
        self.version = Some(version);
        self
    }

    pub fn about(mut self, about: &'static str) -> Self {
        self.about = Some(about);
        self
    }

    pub fn styles(mut self, styles: Styles) -> Self {
        self.styles = styles;
        self
    }

    pub fn arg(mut self, arg: Arg) -> Self {
        self.args.push(arg);
        self
    }

    pub fn args<I: IntoIterator<Item = Arg>>(mut self, args: I) -> Self {
        self.args.extend(args);
        self
    }

    pub fn subcommand(mut self, sub: Ramp) -> Self {
        self.subcommands.push(sub);
        self
    }

    // --- Parser Core ---

    /// Parse `std::env::args()` or exit printing styled errors/help.
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

    /// Parse from any custom string slice.
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
        while i < tokens.len() {
            let token = &tokens[i];

            // Subcommands
            if !token.starts_with('-') && positional_idx == 0 {
                if let Some(sub) = self.subcommands.iter().find(|s| s.name == token) {
                    let sub_matches = sub.clone().try_get_matches_from(&tokens[i..])?;
                    matches.subcommand = Some((sub.name.to_string(), Box::new(sub_matches)));
                    return Ok(matches);
                }
            }

            // Flags / Options
            if token == "-h" || token == "--help" {
                self.print_help();
                process::exit(0);
            } else if token == "-V" || token == "--version" {
                self.print_version();
                process::exit(0);
            } else if token.starts_with("--") {
                let long_name = &token[2..];
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
            } else if token.starts_with('-') && token.len() > 1 {
                let short_char = token.chars().nth(1).unwrap();
                let matched_arg = effective_args
                    .iter()
                    .find(|a| a.short == Some(short_char))
                    .ok_or_else(|| {
                        self.format_error(&format!("unexpected argument '-{short_char}' found"))
                    })?;

                match matched_arg.action {
                    ArgAction::SetTrue => {
                        matches.flags.insert(matched_arg.id.to_string());
                    }
                    ArgAction::Set | ArgAction::Append => {
                        i += 1;
                        if i >= tokens.len() {
                            return Err(self.format_error(&format!(
                                "argument '-{short_char}' requires a value"
                            )));
                        }
                        matches
                            .values
                            .entry(matched_arg.id.to_string())
                            .or_default()
                            .push(tokens[i].clone());
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

    // --- Formatters and Display ---

    pub fn print_version(&self) {
        let v = self.version.unwrap_or("unknown");
        println!(
            "{} {}",
            self.styles.literal.paint(self.name),
            self.styles.placeholder.paint(v)
        );
    }

    pub fn print_help(&self) {
        use crate::color::Color;
        use crate::effect::Effects;
        use crate::style::Style;

        // --- Color & Style Palette ---
        // Headers & Badges (Background + Invert + Bold)
        let bar_accent = Style::new().bold().fg(Color::BrightCyan);
        let badge_name = Style::new().bold().bg(Color::Teal).fg(Color::Black);
        let ver_pill = Style::new().italic().fg(Color::Gold);
        let about_txt = Style::new().fg(Color::Silver);

        // Sections (Underline + Bold + Orange/Purple)
        let sec_arrow = Style::new().bold().fg(Color::Orange);
        let sec_title = Style::new().bold().underline().fg(Color::BrightWhite);

        // Syntax & Tokens
        let prompt_glyph = Style::new().bold().fg(Color::BrightMagenta);
        let app_cmd = Style::new().bold().fg(Color::White);
        let subcmd_syntax = Style::new().bold().fg(Color::Lime);
        let pos_syntax = Style::new().italic().fg(Color::Gold);
        let opt_syntax = Style::new().dim().fg(Color::BrightCyan);

        // Items (Commands, Arguments, Options)
        let cmd_item = Style::new().bold().fg(Color::Lime);
        let arg_item = Style::new().italic().fg(Color::Gold);
        let opt_flag = Style::new().bold().fg(Color::BrightCyan);
        let opt_val = Style::new().fg(Color::Teal);
        let item_help = Style::new().fg(Color::BrightWhite);

        // Metadata Tags
        let req_tag = Style::new().bold().fg(Color::Pink);
        let opt_tag = Style::new().dim().fg(Color::BrightBlack);
        let def_tag = Style::new().fg(Color::Teal).dim();

        // Footer Card (Invert + Dim)
        let tip_badge = Style::new().bold().bg(Color::Indigo).fg(Color::BrightWhite);
        let tip_text = Style::new().dim().fg(Color::Silver);

        println!();

        // 1. Accent Bar Header & About
        let v_str = self.version.unwrap_or("0.1.0");
        let name_upper = self.name.to_uppercase();

        print!(
            "{} {}  {} ",
            bar_accent.paint("┃"),
            badge_name.paint(&format!(" {name_upper} ")),
            ver_pill.paint(&format!("v{v_str}"))
        );

        if let Some(about) = self.about {
            print!(" {}", about_txt.paint(about));
        }
        println!("\n{}", bar_accent.paint("┃"));

        // 2. Syntax Blueprint
        print!(
            "{} {} {}",
            bar_accent.paint("┃"),
            prompt_glyph.paint("$"),
            app_cmd.paint(self.name)
        );

        if self.args.iter().any(|a| !a.is_positional()) {
            print!(" {}", opt_syntax.paint("[OPTIONS]"));
        }
        for pos in self.args.iter().filter(|a| a.is_positional()) {
            let val = pos.value_name.unwrap_or(pos.id);
            if pos.required {
                print!(" {}", pos_syntax.paint(&format!("<{val}>")));
            } else {
                print!(" {}", opt_syntax.paint(&format!("[{val}]")));
            }
        }
        if !self.subcommands.is_empty() {
            print!(" {}", subcmd_syntax.paint("[COMMAND]"));
        }
        println!("\n");

        // 3. Subcommands Section
        if !self.subcommands.is_empty() {
            println!("{} {}", sec_arrow.paint("›"), sec_title.paint("Commands"));
            for cmd in &self.subcommands {
                let about = cmd.about.unwrap_or("");
                println!(
                    "  {: <16}  {}",
                    cmd_item.paint(cmd.name),
                    item_help.paint(about)
                );
            }
            println!();
        }

        // 4. Positional Arguments Section
        let positionals: Vec<&Arg> = self.args.iter().filter(|a| a.is_positional()).collect();
        if !positionals.is_empty() {
            println!("{} {}", sec_arrow.paint("›"), sec_title.paint("Arguments"));
            for arg in positionals {
                let name = arg.value_name.unwrap_or(arg.id);
                let help = arg.help.unwrap_or("");
                let status = if arg.required {
                    format!("{}", req_tag.paint("[required]"))
                } else {
                    format!("{}", opt_tag.paint("[optional]"))
                };

                println!(
                    "  {: <16}  {: <44} {}",
                    arg_item.paint(&format!("<{name}>")),
                    item_help.paint(help),
                    status
                );
            }
            println!();
        }

        // 5. Options & Flags Section
        let options: Vec<&Arg> = self.args.iter().filter(|a| !a.is_positional()).collect();
        println!(
            "{} {}",
            sec_arrow.paint("›"),
            sec_title.paint("Flags & Options")
        );

        for arg in &options {
            let mut syntax = String::new();
            if let Some(s) = arg.short {
                syntax.push_str(&format!("-{s}, "));
            } else {
                syntax.push_str("    ");
            }
            if let Some(l) = arg.long {
                syntax.push_str(&format!("--{l}"));
            }

            let rendered_flag = opt_flag.paint(&syntax);
            let val_suffix = if arg.action != ArgAction::SetTrue {
                format!(
                    " {}",
                    opt_val.paint(&format!("<{}>", arg.value_name.unwrap_or("VAL")))
                )
            } else {
                String::new()
            };

            let combined_flag = format!("{rendered_flag}{val_suffix}");
            let help = arg.help.unwrap_or("");
            let default_note = if let Some(def) = arg.default_value {
                format!("{}", def_tag.paint(&format!("(default: \"{def}\")")))
            } else {
                String::new()
            };

            println!(
                "  {: <24}  {: <40} {}",
                combined_flag,
                item_help.paint(help),
                default_note
            );
        }

        // Built-in Defaults
        if !options
            .iter()
            .any(|a| a.short == Some('h') || a.long == Some("help"))
        {
            println!(
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
            println!(
                "  {: <24}  {}",
                opt_flag.paint("  -V, --version"),
                item_help.paint("Show version number")
            );
        }

        // 6. Subcommand Hint Footer
        if !self.subcommands.is_empty() {
            println!();
            println!(
                "  {} {} {}{}",
                tip_badge.paint(" TIP "),
                tip_text.paint("Run"),
                opt_flag.paint(&format!("{} <command> --help", self.name)),
                tip_text.paint(" for details on a specific subcommand.")
            );
        }
        println!();
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
