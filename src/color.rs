//! src/color.rs
//! part of ramp. A Colorful and Customisable CLI wrapper with default for quick start!
use crate::style::{Style, Styled};
use core::fmt::{self, Formatter};
/// 24 distinct named colors, 8-bit ANSI palette, and 24-bit TrueColor (RGB).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    // Standard 8 ANSI Colors (Codes 30-37 / 40-47)
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,

    // 8 Bright / High-Intensity ANSI Colors (Codes 90-97 / 100-107)
    BrightBlack, // Gray / Charcoal
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,

    // 8 Extended Named Palette Colors (Standard 8-bit mappings)
    Orange, // 256 Code: 208
    Purple, // 256 Code: 129
    Pink,   // 256 Code: 205
    Teal,   // 256 Code: 30
    Gold,   // 256 Code: 220
    Silver, // 256 Code: 248
    Lime,   // 256 Code: 118
    Indigo, // 256 Code: 54

    // Full Gamut Extensibility
    Ansi256(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    /// Render color parameters to a standard ANSI sequence stream.
    pub fn write_ansi(&self, f: &mut Formatter<'_>, is_bg: bool) -> fmt::Result {
        let (base_std, base_bright, base_ext) = if is_bg {
            (40u8, 100u8, 48u8)
        } else {
            (30u8, 90u8, 38u8)
        };

        match self {
            Self::Black => write!(f, "{}", base_std),
            Self::Red => write!(f, "{}", base_std + 1),
            Self::Green => write!(f, "{}", base_std + 2),
            Self::Yellow => write!(f, "{}", base_std + 3),
            Self::Blue => write!(f, "{}", base_std + 4),
            Self::Magenta => write!(f, "{}", base_std + 5),
            Self::Cyan => write!(f, "{}", base_std + 6),
            Self::White => write!(f, "{}", base_std + 7),

            Self::BrightBlack => write!(f, "{}", base_bright),
            Self::BrightRed => write!(f, "{}", base_bright + 1),
            Self::BrightGreen => write!(f, "{}", base_bright + 2),
            Self::BrightYellow => write!(f, "{}", base_bright + 3),
            Self::BrightBlue => write!(f, "{}", base_bright + 4),
            Self::BrightMagenta => write!(f, "{}", base_bright + 5),
            Self::BrightCyan => write!(f, "{}", base_bright + 6),
            Self::BrightWhite => write!(f, "{}", base_bright + 7),

            Self::Orange => write!(f, "{base_ext};5;208"),
            Self::Purple => write!(f, "{base_ext};5;129"),
            Self::Pink => write!(f, "{base_ext};5;205"),
            Self::Teal => write!(f, "{base_ext};5;30"),
            Self::Gold => write!(f, "{base_ext};5;220"),
            Self::Silver => write!(f, "{base_ext};5;248"),
            Self::Lime => write!(f, "{base_ext};5;118"),
            Self::Indigo => write!(f, "{base_ext};5;54"),

            Self::Ansi256(code) => write!(f, "{base_ext};5;{code}"),
            Self::Rgb(r, g, b) => write!(f, "{base_ext};2;{r};{g};{b}"),
        }
    }
}

// Public Constants (Presets)

pub const RESET: &str = "\x1b[0m";

pub const BOLD: Style = Style::new().bold();
pub const DIM: Style = Style::new().dim();
pub const ITALIC: Style = Style::new().italic();
pub const UNDERLINE: Style = Style::new().underline();

pub const BLACK: Style = Style::new().fg(Color::Black);
pub const RED: Style = Style::new().fg(Color::Red);
pub const GREEN: Style = Style::new().fg(Color::Green);
pub const YELLOW: Style = Style::new().fg(Color::Yellow);
pub const BLUE: Style = Style::new().fg(Color::Blue);
pub const MAGENTA: Style = Style::new().fg(Color::Magenta);
pub const CYAN: Style = Style::new().fg(Color::Cyan);
pub const WHITE: Style = Style::new().fg(Color::White);

pub const BRIGHT_BLACK: Style = Style::new().fg(Color::BrightBlack);
pub const BRIGHT_RED: Style = Style::new().fg(Color::BrightRed);
pub const BRIGHT_GREEN: Style = Style::new().fg(Color::BrightGreen);
pub const BRIGHT_YELLOW: Style = Style::new().fg(Color::BrightYellow);
pub const BRIGHT_BLUE: Style = Style::new().fg(Color::BrightBlue);
pub const BRIGHT_MAGENTA: Style = Style::new().fg(Color::BrightMagenta);
pub const BRIGHT_CYAN: Style = Style::new().fg(Color::BrightCyan);
pub const BRIGHT_WHITE: Style = Style::new().fg(Color::BrightWhite);

pub const ORANGE: Style = Style::new().fg(Color::Orange);
pub const PURPLE: Style = Style::new().fg(Color::Purple);
pub const PINK: Style = Style::new().fg(Color::Pink);
pub const TEAL: Style = Style::new().fg(Color::Teal);
pub const GOLD: Style = Style::new().fg(Color::Gold);
pub const SILVER: Style = Style::new().fg(Color::Silver);
pub const LIME: Style = Style::new().fg(Color::Lime);
pub const INDIGO: Style = Style::new().fg(Color::Indigo);
