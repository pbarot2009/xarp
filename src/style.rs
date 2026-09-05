use crate::color::Color;
use crate::effect::Effects;
use core::fmt::{self, Display, Formatter};
use core::ops::BitOr;

/// Represents a combined text style (Effects + Foreground + Background).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style {
    /// Foreground color.
    pub fg: Option<Color>,
    /// Background color.
    pub bg: Option<Color>,
    /// Set of visual text effects.
    pub effects: Effects,
}

impl Style {
    /// Creates a blank/unstyled style.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            effects: Effects::NONE,
        }
    }

    /// Returns `true` if no color or text effects are applied.
    #[must_use]
    #[inline]
    pub const fn is_plain(&self) -> bool {
        self.fg.is_none() && self.bg.is_none() && self.effects.is_empty()
    }

    // --- Fluent Builders ---

    /// Sets the foreground color.
    #[must_use]
    #[inline]
    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Sets the background color.
    #[must_use]
    #[inline]
    pub const fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Adds text effects to the current style.
    #[must_use]
    #[inline]
    pub const fn effects(mut self, effects: Effects) -> Self {
        self.effects = self.effects.insert(effects);
        self
    }

    /// Applies bold styling.
    #[must_use]
    #[inline]
    pub const fn bold(self) -> Self {
        self.effects(Effects::BOLD)
    }

    /// Applies dim (faint) styling.
    #[must_use]
    #[inline]
    pub const fn dim(self) -> Self {
        self.effects(Effects::DIM)
    }

    /// Applies italic styling.
    #[must_use]
    #[inline]
    pub const fn italic(self) -> Self {
        self.effects(Effects::ITALIC)
    }

    /// Applies single underline styling.
    #[must_use]
    #[inline]
    pub const fn underline(self) -> Self {
        self.effects(Effects::UNDERLINE)
    }

    /// Applies slow blink styling.
    #[must_use]
    #[inline]
    pub const fn blink(self) -> Self {
        self.effects(Effects::BLINK)
    }

    /// Inverts foreground and background colors.
    #[must_use]
    #[inline]
    pub const fn invert(self) -> Self {
        self.effects(Effects::INVERT)
    }

    /// Hides the text.
    #[must_use]
    #[inline]
    pub const fn hidden(self) -> Self {
        self.effects(Effects::HIDDEN)
    }

    /// Applies strikethrough (crossed-out) styling.
    #[must_use]
    #[inline]
    pub const fn strikethrough(self) -> Self {
        self.effects(Effects::STRIKETHROUGH)
    }

    /// Applies double underline styling.
    #[must_use]
    #[inline]
    pub const fn double_underline(self) -> Self {
        self.effects(Effects::DOUBLE_UNDERLINE)
    }

    /// Wraps any `Display` value in a struct that automatically applies this style
    /// and resets formatting afterwards.
    ///
    /// Takes `self` by value (`Style` is `Copy`), so temporaries can be
    /// painted directly: `Style::new().bold().paint("hi")`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use xarp::style::Style;
    /// use xarp::color::Color;
    ///
    /// let styled = Style::new().bold().fg(Color::Red).paint("alert");
    /// assert!(styled.to_string().contains("alert"));
    /// ```
    #[must_use]
    #[inline]
    pub fn paint<T: Display + ?Sized>(self, target: &T) -> Styled<'_, T> {
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

/// Merges two styles: the right-hand side `fg`/`bg` win when set, and effects
/// are combined with a union.
///
/// # Example
///
/// ```rust
/// use xarp::style::Style;
/// use xarp::color::Color;
/// use xarp::effect::Effects;
///
/// let merged = Style::new().fg(Color::Red) | Style::new().bold();
/// assert_eq!(merged.fg, Some(Color::Red));
/// assert!(merged.effects.contains(Effects::BOLD));
/// ```
impl BitOr<Style> for Style {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self {
            fg: rhs.fg.or(self.fg),
            bg: rhs.bg.or(self.bg),
            effects: self.effects | rhs.effects,
        }
    }
}

/// Combines effects with a style: effects are unioned, the style's
/// foreground and background colors are kept.
impl BitOr<Style> for Effects {
    type Output = Style;
    #[inline]
    fn bitor(self, rhs: Style) -> Style {
        Style::new().effects(self) | rhs
    }
}

/// Combines a color with a style: the style's foreground wins when set,
/// otherwise the color becomes the foreground; backgrounds and effects merge.
impl BitOr<Style> for Color {
    type Output = Style;
    #[inline]
    fn bitor(self, rhs: Style) -> Style {
        Style::new().fg(self) | rhs
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
///
/// Owns a copy of the style (`Style` is `Copy`), so painted values from
/// temporaries can be bound to variables.
pub struct Styled<'a, T: ?Sized> {
    style: Style,
    target: &'a T,
}

impl<T: Display + ?Sized> Display for Styled<'_, T> {
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
    /// Style applied to section headers.
    pub header: Style,
    /// Style applied to usage instructions.
    pub usage: Style,
    /// Style applied to command literals and flags.
    pub literal: Style,
    /// Style applied to argument value placeholders.
    pub placeholder: Style,
    /// Style applied to error messages.
    pub error: Style,
    /// Style applied to valid values or states.
    pub valid: Style,
    /// Style applied to invalid values or warnings.
    pub invalid: Style,
    /// Style applied to warning messages.
    pub warning: Style,
    /// Style applied to secondary or dimmed text.
    pub muted: Style,
}

impl Styles {
    /// Returns uncolored / plain styles.
    #[must_use]
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
    ///
    /// Automatically returns plain styles when the `NO_COLOR` environment
    /// variable is present, including when it is set to an empty string
    /// (any presence disables color, per <https://no-color.org>).
    #[must_use]
    pub fn styled() -> Self {
        if std::env::var_os("NO_COLOR").is_some() {
            return Self::plain();
        }

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

    /// Sets the header style.
    #[must_use]
    pub const fn header(mut self, style: Style) -> Self {
        self.header = style;
        self
    }

    /// Sets the usage style.
    #[must_use]
    pub const fn usage(mut self, style: Style) -> Self {
        self.usage = style;
        self
    }

    /// Sets the literal style.
    #[must_use]
    pub const fn literal(mut self, style: Style) -> Self {
        self.literal = style;
        self
    }

    /// Sets the placeholder style.
    #[must_use]
    pub const fn placeholder(mut self, style: Style) -> Self {
        self.placeholder = style;
        self
    }

    /// Sets the error style.
    #[must_use]
    pub const fn error(mut self, style: Style) -> Self {
        self.error = style;
        self
    }

    /// Sets the valid style.
    #[must_use]
    pub const fn valid(mut self, style: Style) -> Self {
        self.valid = style;
        self
    }

    /// Sets the invalid style.
    #[must_use]
    pub const fn invalid(mut self, style: Style) -> Self {
        self.invalid = style;
        self
    }

    /// Sets the warning style.
    #[must_use]
    pub const fn warning(mut self, style: Style) -> Self {
        self.warning = style;
        self
    }

    /// Sets the muted style.
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_temporary_can_be_bound() {
        let painted = Style::new().bold().paint("hi");
        assert!(painted.to_string().contains("hi"));
    }

    #[test]
    fn style_merge_prefers_right_colors_and_unions_effects() {
        let merged = Style::new().fg(Color::Red) | Style::new().bold();
        assert_eq!(merged.fg, Some(Color::Red));
        assert!(merged.effects.contains(Effects::BOLD));

        let overridden = Style::new().fg(Color::Red) | Style::new().fg(Color::Blue);
        assert_eq!(overridden.fg, Some(Color::Blue));
    }

    #[test]
    fn effects_and_colors_merge_with_styles() {
        let from_effects = Effects::BOLD | Style::new().fg(Color::Red);
        assert_eq!(from_effects.fg, Some(Color::Red));
        assert!(from_effects.effects.contains(Effects::BOLD));

        let from_color = Color::Red | Style::new().bold();
        assert_eq!(from_color.fg, Some(Color::Red));
        assert!(from_color.effects.contains(Effects::BOLD));
    }
}
