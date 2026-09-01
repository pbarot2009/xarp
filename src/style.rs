// === Colors ===
// 24 Named Presets
// 256-Color & 24-bit TrueColor

pub enum Color {
    //Standard 8 ANSI Colors
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,

    // 8 Bright / High-Intensity ANSI Colors
    BrightBlack, // Gray / Charcoal
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,

    // 8 Extended Named Palette Colors
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
