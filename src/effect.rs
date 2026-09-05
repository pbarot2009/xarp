use core::fmt::{self, Formatter};
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

/// ANSI text style modifiers and font effects.
///
/// The inner value is a bitmask; only the ten documented flag bits are valid.
/// Use [`Effects::from_bits`] to fallibly convert a raw mask.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Effects(
    /// Raw bitflags representation. Only bits covered by [`Effects::ALL`] are defined.
    pub u16,
);

/// Formats the names of the set effects (e.g. `Effects(BOLD | UNDERLINE)`).
///
/// An empty set formats as `Effects(NONE)`. Unknown bits (outside
/// [`Effects::ALL`]) are reported as a hexadecimal mask.
impl fmt::Debug for Effects {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "Effects(NONE)");
        }
        write!(f, "Effects(")?;
        let mut first = true;
        let mut named = |f: &mut Formatter<'_>, name: &str, flag: Self| -> fmt::Result {
            if self.contains(flag) {
                if !first {
                    write!(f, " | ")?;
                }
                first = false;
                write!(f, "{name}")?;
            }
            Ok(())
        };
        named(f, "BOLD", Self::BOLD)?;
        named(f, "DIM", Self::DIM)?;
        named(f, "ITALIC", Self::ITALIC)?;
        named(f, "UNDERLINE", Self::UNDERLINE)?;
        named(f, "BLINK", Self::BLINK)?;
        named(f, "RAPID_BLINK", Self::RAPID_BLINK)?;
        named(f, "INVERT", Self::INVERT)?;
        named(f, "HIDDEN", Self::HIDDEN)?;
        named(f, "STRIKETHROUGH", Self::STRIKETHROUGH)?;
        named(f, "DOUBLE_UNDERLINE", Self::DOUBLE_UNDERLINE)?;
        let unknown = self.0 & !Self::ALL.0;
        if unknown != 0 {
            if !first {
                write!(f, " | ")?;
            }
            write!(f, "0x{unknown:X}")?;
        }
        write!(f, ")")
    }
}

impl Effects {
    /// No text effects.
    pub const NONE: Self = Self(0);

    /// Bold or increased intensity.
    pub const BOLD: Self = Self(1 << 0);

    /// Dim or decreased intensity.
    pub const DIM: Self = Self(1 << 1);

    /// Italic text.
    pub const ITALIC: Self = Self(1 << 2);

    /// Single underline.
    pub const UNDERLINE: Self = Self(1 << 3);

    /// Slow blinking text.
    pub const BLINK: Self = Self(1 << 4);

    /// Rapid blinking text.
    pub const RAPID_BLINK: Self = Self(1 << 5);

    /// Inverted foreground and background colors.
    pub const INVERT: Self = Self(1 << 6);

    /// Hidden or concealed text.
    pub const HIDDEN: Self = Self(1 << 7);

    /// Strikethrough or crossed-out text.
    pub const STRIKETHROUGH: Self = Self(1 << 8);

    /// Double underline.
    ///
    /// Encoded as SGR code 21. Note that some terminals interpret code 21 as
    /// "bold off" instead of double underline, so this effect is less portable
    /// than the others.
    pub const DOUBLE_UNDERLINE: Self = Self(1 << 9);

    /// Mask of all defined effect bits.
    pub const ALL: Self = Self(
        Self::BOLD.0
            | Self::DIM.0
            | Self::ITALIC.0
            | Self::UNDERLINE.0
            | Self::BLINK.0
            | Self::RAPID_BLINK.0
            | Self::INVERT.0
            | Self::HIDDEN.0
            | Self::STRIKETHROUGH.0
            | Self::DOUBLE_UNDERLINE.0,
    );

    /// Creates a set containing every defined effect.
    #[must_use]
    #[inline]
    pub const fn all() -> Self {
        Self::ALL
    }

    /// Converts a raw bitmask into an effect set.
    ///
    /// Returns `None` when `bits` contains undefined bits (outside
    /// [`Effects::ALL`]). Use [`Effects::from_bits_truncate`] to discard
    /// undefined bits instead.
    #[must_use]
    #[inline]
    pub const fn from_bits(bits: u16) -> Option<Self> {
        if bits & !Self::ALL.0 == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    /// Converts a raw bitmask into an effect set, discarding undefined bits.
    #[must_use]
    #[inline]
    pub const fn from_bits_truncate(bits: u16) -> Self {
        Self(bits & Self::ALL.0)
    }

    /// Creates an empty set of effects.
    #[must_use]
    #[inline]
    pub const fn empty() -> Self {
        Self::NONE
    }

    /// Returns `true` if no effects are set.
    #[must_use]
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all effects in `other` are contained in `self`.
    #[must_use]
    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Inserts the specified effects into `self`.
    #[must_use]
    #[inline]
    pub const fn insert(mut self, other: Self) -> Self {
        self.0 |= other.0;
        self
    }

    /// Removes the specified effects from `self`.
    #[must_use]
    #[inline]
    pub const fn remove(mut self, other: Self) -> Self {
        self.0 &= !other.0;
        self
    }
}

impl BitOr for Effects {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Effects {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for Effects {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Effects {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for Effects {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        Self(!self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_lists_set_flag_names() {
        assert_eq!(format!("{:?}", Effects::NONE), "Effects(NONE)");
        assert_eq!(format!("{:?}", Effects::BOLD), "Effects(BOLD)");
        assert_eq!(
            format!("{:?}", Effects::BOLD | Effects::UNDERLINE),
            "Effects(BOLD | UNDERLINE)"
        );
    }

    #[test]
    fn all_and_from_bits_round_trip() {
        let all = Effects::all();
        assert!(all.contains(Effects::BOLD));
        assert!(all.contains(Effects::DOUBLE_UNDERLINE));
        assert_eq!(Effects::from_bits(all.0), Some(all));
        assert_eq!(Effects::from_bits(u16::MAX), None);
        assert_eq!(
            Effects::from_bits_truncate(u16::MAX),
            Effects::from_bits_truncate(Effects::ALL.0)
        );
    }
}
