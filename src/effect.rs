// === Effects / Text Attributes (Bitflags) ===

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

/// ANSI text style modifiers and font effects.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Effects(pub u16);

impl Effects {
    pub const NONE: Self = Self(0);
    pub const BOLD: Self = Self(1 << 0);
    pub const DIM: Self = Self(1 << 1);
    pub const ITALIC: Self = Self(1 << 2);
    pub const UNDERLINE: Self = Self(1 << 3);
    pub const BLINK: Self = Self(1 << 4);
    pub const RAPID_BLINK: Self = Self(1 << 5);
    pub const INVERT: Self = Self(1 << 6);
    pub const HIDDEN: Self = Self(1 << 7);
    pub const STRIKETHROUGH: Self = Self(1 << 8);
    pub const DOUBLE_UNDERLINE: Self = Self(1 << 9);

    #[inline]
    pub const fn empty() -> Self {
        Self::NONE
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn insert(mut self, other: Self) -> Self {
        self.0 |= other.0;
        self
    }

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
