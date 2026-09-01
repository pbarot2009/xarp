use crate::color::Color;
use crate::effect::Effects;
use core::fmt::{self, Display, Formatter};
use core::ops::BitOr;

/// Represents a combined text style (Effects + Foreground + Background).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub effects: Effects,
}

impl Style {
    /// Creates a blank/unstyled style.
    #[inline]
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            effects: Effects::NONE,
        }
    }

    #[inline]
    pub const fn is_plain(&self) -> bool {
        self.fg.is_none() && self.bg.is_none() && self.effects.is_empty()
    }

    // --- Fluent Builders ---

    #[inline]
    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    #[inline]
    pub const fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    #[inline]
    pub const fn effects(mut self, effects: Effects) -> Self {
        self.effects = self.effects.insert(effects);
        self
    }

    #[inline]
    pub const fn bold(self) -> Self {
        self.effects(Effects::BOLD)
    }

    #[inline]
    pub const fn dim(self) -> Self {
        self.effects(Effects::DIM)
    }

    #[inline]
    pub const fn italic(self) -> Self {
        self.effects(Effects::ITALIC)
    }

    #[inline]
    pub const fn underline(self) -> Self {
        self.effects(Effects::UNDERLINE)
    }

    #[inline]
    pub const fn blink(self) -> Self {
        self.effects(Effects::BLINK)
    }

    #[inline]
    pub const fn invert(self) -> Self {
        self.effects(Effects::INVERT)
    }

    #[inline]
    pub const fn hidden(self) -> Self {
        self.effects(Effects::HIDDEN)
    }

    #[inline]
    pub const fn strikethrough(self) -> Self {
        self.effects(Effects::STRIKETHROUGH)
    }

    #[inline]
    pub const fn double_underline(self) -> Self {
        self.effects(Effects::DOUBLE_UNDERLINE)
    }

    /// Wraps any `Display` value in a struct that automatically applies this style
    /// and resets formatting afterwards.
    #[inline]
    pub fn paint<'a, T: Display + ?Sized>(&'a self, target: &'a T) -> Styled<'a, T> {
        Styled {
            style: self,
            target,
        }
    }
}

// Ergonomic Operator Overloading
impl BitOr<Effects> for Style {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Effects) -> Self {
        self.effects(rhs)
    }
}

impl BitOr<Color> for Style {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Color) -> Self {
        self.fg(rhs)
    }
}

impl BitOr<Effects> for Color {
    type Output = Style;
    #[inline]
    fn bitor(self, rhs: Effects) -> Style {
        Style::new().fg(self).effects(rhs)
    }
}

impl BitOr<Color> for Effects {
    type Output = Style;
    #[inline]
    fn bitor(self, rhs: Color) -> Style {
        Style::new().fg(rhs).effects(self)
    }
}

// === ANSI Output Formatter ===

impl Display for Style {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Alternate format `{style:#}` renders the ANSI Reset code
        if f.alternate() {
            return write!(f, "\x1b[0m");
        }

        if self.is_plain() {
            return Ok(());
        }

        write!(f, "\x1b[")?;
        let mut first = true;

        let mut write_effect = |f: &mut Formatter<'_>, code: u8| -> fmt::Result {
            if !first {
                write!(f, ";")?;
            }
            first = false;
            write!(f, "{code}")
        };

        if self.effects.contains(Effects::BOLD) {
            write_effect(f, 1)?;
        }
        if self.effects.contains(Effects::DIM) {
            write_effect(f, 2)?;
        }
        if self.effects.contains(Effects::ITALIC) {
            write_effect(f, 3)?;
        }
        if self.effects.contains(Effects::UNDERLINE) {
            write_effect(f, 4)?;
        }
        if self.effects.contains(Effects::BLINK) {
            write_effect(f, 5)?;
        }
        if self.effects.contains(Effects::RAPID_BLINK) {
            write_effect(f, 6)?;
        }
        if self.effects.contains(Effects::INVERT) {
            write_effect(f, 7)?;
        }
        if self.effects.contains(Effects::HIDDEN) {
            write_effect(f, 8)?;
        }
        if self.effects.contains(Effects::STRIKETHROUGH) {
            write_effect(f, 9)?;
        }
        if self.effects.contains(Effects::DOUBLE_UNDERLINE) {
            write_effect(f, 21)?;
        }

        if let Some(fg) = self.fg {
            if !first {
                write!(f, ";")?;
            }
            first = false;
            fg.write_ansi(f, false)?;
        }

        if let Some(bg) = self.bg {
            if !first {
                write!(f, ";")?;
            }
            bg.write_ansi(f, true)?;
        }

        write!(f, "m")
    }
}

/// Helper struct returned by `Style::paint`.
pub struct Styled<'a, T: ?Sized> {
    style: &'a Style,
    target: &'a T,
}

impl<'a, T: Display + ?Sized> Display for Styled<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.style.is_plain() {
            self.target.fmt(f)
        } else {
            write!(f, "{}{}{:#}", self.style, self.target, self.style)
        }
    }
}

// === CLI Styles / Theme Definition (Clap Alternative Core) ===

/// Styling configuration for CLI argument parsers (Help messages, Usage, Errors).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Styles {
    pub header: Style,
    pub usage: Style,
    pub literal: Style,
    pub placeholder: Style,
    pub error: Style,
    pub valid: Style,
    pub invalid: Style,
    pub warning: Style,
    pub muted: Style,
}

impl Styles {
    /// Returns uncolored / plain styles.
    pub const fn plain() -> Self {
        Self {
            header: Style::new(),
            usage: Style::new(),
            literal: Style::new(),
            placeholder: Style::new(),
            error: Style::new(),
            valid: Style::new(),
            invalid: Style::new(),
            warning: Style::new(),
            muted: Style::new(),
        }
    }

    /// Default styled CLI theme (Cargo / Clap v4 styled look).
    pub const fn styled() -> Self {
        Self {
            header: Style::new().bold().underline(),
            usage: Style::new().bold().underline(),
            literal: Style::new().bold().fg(Color::BrightCyan),
            placeholder: Style::new().fg(Color::Cyan),
            error: Style::new().bold().fg(Color::BrightRed),
            valid: Style::new().bold().fg(Color::BrightGreen),
            invalid: Style::new().bold().fg(Color::BrightYellow),
            warning: Style::new().bold().fg(Color::Yellow),
            muted: Style::new().dim(),
        }
    }

    pub const fn header(mut self, style: Style) -> Self {
        self.header = style;
        self
    }
    pub const fn usage(mut self, style: Style) -> Self {
        self.usage = style;
        self
    }
    pub const fn literal(mut self, style: Style) -> Self {
        self.literal = style;
        self
    }
    pub const fn placeholder(mut self, style: Style) -> Self {
        self.placeholder = style;
        self
    }
    pub const fn error(mut self, style: Style) -> Self {
        self.error = style;
        self
    }
    pub const fn valid(mut self, style: Style) -> Self {
        self.valid = style;
        self
    }
    pub const fn invalid(mut self, style: Style) -> Self {
        self.invalid = style;
        self
    }
    pub const fn warning(mut self, style: Style) -> Self {
        self.warning = style;
        self
    }
    pub const fn muted(mut self, style: Style) -> Self {
        self.muted = style;
        self
    }
}

impl Default for Styles {
    fn default() -> Self {
        Self::styled()
    }
}
