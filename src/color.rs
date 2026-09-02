//! Color definitions, ANSI escape sequence generation, and color style presets.

use crate::style::Style;
use core::fmt::{self, Formatter};

/// 24 distinct named colors, 8-bit ANSI palette, and 24-bit `TrueColor` (RGB).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    /// Standard ANSI black.
    Black,
    /// Standard ANSI red.
    Red,
    /// Standard ANSI green.
    Green,
    /// Standard ANSI yellow.
    Yellow,
    /// Standard ANSI blue.
    Blue,
    /// Standard ANSI magenta.
    Magenta,
    /// Standard ANSI cyan.
    Cyan,
    /// Standard ANSI white.
    White,

    /// High-intensity black (charcoal / gray).
    BrightBlack,
    /// High-intensity red.
    BrightRed,
    /// High-intensity green.
    BrightGreen,
    /// High-intensity yellow.
    BrightYellow,
    /// High-intensity blue.
    BrightBlue,
    /// High-intensity magenta.
    BrightMagenta,
    /// High-intensity cyan.
    BrightCyan,
    /// High-intensity white.
    BrightWhite,

    /// Extended palette orange (ANSI 208).
    Orange,
    /// Extended palette purple (ANSI 129).
    Purple,
    /// Extended palette pink (ANSI 205).
    Pink,
    /// Extended palette teal (ANSI 30).
    Teal,
    /// Extended palette gold (ANSI 220).
    Gold,
    /// Extended palette silver (ANSI 248).
    Silver,
    /// Extended palette lime (ANSI 118).
    Lime,
    /// Extended palette indigo (ANSI 54).
    Indigo,

    /// 8-bit ANSI 256-color palette code.
    Ansi256(u8),
    /// 24-bit `TrueColor` RGB values (red, green, blue).
    Rgb(u8, u8, u8),
}

impl Color {
    /// Renders color parameters to a standard ANSI sequence stream.
    pub fn write_ansi(&self, f: &mut Formatter<'_>, is_bg: bool) -> fmt::Result {
        let (base_std, base_bright, base_ext) = if is_bg {
            (40u8, 100u8, 48u8)
        } else {
            (30u8, 90u8, 38u8)
        };

        match self {
            Self::Black => write!(f, "{base_std}"),
            Self::Red => write!(f, "{}", base_std + 1),
            Self::Green => write!(f, "{}", base_std + 2),
            Self::Yellow => write!(f, "{}", base_std + 3),
            Self::Blue => write!(f, "{}", base_std + 4),
            Self::Magenta => write!(f, "{}", base_std + 5),
            Self::Cyan => write!(f, "{}", base_std + 6),
            Self::White => write!(f, "{}", base_std + 7),

            Self::BrightBlack => write!(f, "{base_bright}"),
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

/// ANSI escape sequence to reset terminal styling and colors.
pub const RESET: &str = "\x1b[0m";

/// Preset style with bold text effect.
pub const BOLD: Style = Style::new().bold();
/// Preset style with dim (faint) text effect.
pub const DIM: Style = Style::new().dim();
/// Preset style with italic text effect.
pub const ITALIC: Style = Style::new().italic();
/// Preset style with underline text effect.
pub const UNDERLINE: Style = Style::new().underline();

/// Preset style with foreground standard black.
pub const BLACK: Style = Style::new().fg(Color::Black);
/// Preset style with foreground standard red.
pub const RED: Style = Style::new().fg(Color::Red);
/// Preset style with foreground standard green.
pub const GREEN: Style = Style::new().fg(Color::Green);
/// Preset style with foreground standard yellow.
pub const YELLOW: Style = Style::new().fg(Color::Yellow);
/// Preset style with foreground standard blue.
pub const BLUE: Style = Style::new().fg(Color::Blue);
/// Preset style with foreground standard magenta.
pub const MAGENTA: Style = Style::new().fg(Color::Magenta);
/// Preset style with foreground standard cyan.
pub const CYAN: Style = Style::new().fg(Color::Cyan);
/// Preset style with foreground standard white.
pub const WHITE: Style = Style::new().fg(Color::White);

/// Preset style with foreground bright black (gray / charcoal).
pub const BRIGHT_BLACK: Style = Style::new().fg(Color::BrightBlack);
/// Preset style with foreground bright red.
pub const BRIGHT_RED: Style = Style::new().fg(Color::BrightRed);
/// Preset style with foreground bright green.
pub const BRIGHT_GREEN: Style = Style::new().fg(Color::BrightGreen);
/// Preset style with foreground bright yellow.
pub const BRIGHT_YELLOW: Style = Style::new().fg(Color::BrightYellow);
/// Preset style with foreground bright blue.
pub const BRIGHT_BLUE: Style = Style::new().fg(Color::BrightBlue);
/// Preset style with foreground bright magenta.
pub const BRIGHT_MAGENTA: Style = Style::new().fg(Color::BrightMagenta);
/// Preset style with foreground bright cyan.
pub const BRIGHT_CYAN: Style = Style::new().fg(Color::BrightCyan);
/// Preset style with foreground bright white.
pub const BRIGHT_WHITE: Style = Style::new().fg(Color::BrightWhite);

/// Preset style with foreground orange.
pub const ORANGE: Style = Style::new().fg(Color::Orange);
/// Preset style with foreground purple.
pub const PURPLE: Style = Style::new().fg(Color::Purple);
/// Preset style with foreground pink.
pub const PINK: Style = Style::new().fg(Color::Pink);
/// Preset style with foreground teal.
pub const TEAL: Style = Style::new().fg(Color::Teal);
/// Preset style with foreground gold.
pub const GOLD: Style = Style::new().fg(Color::Gold);
/// Preset style with foreground silver.
pub const SILVER: Style = Style::new().fg(Color::Silver);
/// Preset style with foreground lime.
pub const LIME: Style = Style::new().fg(Color::Lime);
/// Preset style with foreground indigo.
pub const INDIGO: Style = Style::new().fg(Color::Indigo);
