use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

/// ANSI text style modifiers and font effects.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Effects(
    /// Raw bitflags representation.
    pub u16,
);

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
    pub const DOUBLE_UNDERLINE: Self = Self(1 << 9);

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
