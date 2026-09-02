//! # Styling & Theming
//!
//! This example demonstrates terminal formatting and coloring capabilities:
//! - Using preset styling constants (`BOLD`, `GREEN`, `RED`, `INDIGO`).
//! - Constructing custom text styles with colors, backgrounds, and text effects.
//! - Operator overloading ergonomics (`Color | Effects`).
//! - 24-bit TrueColor (RGB) and 8-bit ANSI 256-color palettes.
//! - Customizing the help output appearance of `Xarp` using the `Styles` configuration.

use xarp::color::{BOLD, CYAN, Color, GOLD, GREEN, INDIGO, RED, WHITE};
use xarp::effect::Effects;
use xarp::style::{Style, Styles};
use xarp::{Arg, ArgAction, Xarp};

/// Demonstrates terminal styling and custom application theme creation.
fn main() {
    println!("=== 1. Built-in Style Presets ===");
    println!("  {}", GREEN.paint("✔ Build succeeded successfully"));
    println!("  {}", RED.paint("✖ Error: connection reset by peer"));
    println!("  {}", GOLD.paint("⚠ Warning: low disk space remaining"));
    println!("  {}", INDIGO.paint("ℹ Note: database migrations pending"));

    println!("\n=== 2. Fluent Custom Styles ===");
    let badge_style = Style::new().bold().fg(Color::Black).bg(Color::BrightCyan);

    let dim_text = Style::new().dim().italic();

    println!(
        "  {} {}",
        badge_style.paint(" RELEASE "),
        dim_text.paint("v2.4.0 (compiled for x86_64-unknown-linux-gnu)")
    );

    println!("\n=== 3. Bitwise Operator Composition ===");
    // Compose Color with text effect bitflags using the `|` operator
    let critical_style = Color::BrightRed | Effects::BOLD | Effects::UNDERLINE;
    let highlight_style = Color::Lime | Effects::ITALIC;

    println!("  {}", critical_style.paint("CRITICAL FAULT DETECTED"));
    println!(
        "  {}",
        highlight_style.paint("Resource allocation verified")
    );

    println!("\n=== 4. TrueColor (RGB) and Extended 256 Colors ===");
    // 24-bit TrueColor (RGB)
    let sunset_orange = Style::new().fg(Color::Rgb(255, 110, 64)).bold();
    let deep_ocean = Style::new()
        .fg(Color::Rgb(0, 210, 255))
        .bg(Color::Rgb(15, 23, 42));

    // 8-bit ANSI-256 color lookup
    let ansi_256_code = Style::new().fg(Color::Ansi256(141)).bold();

    println!("  {}", sunset_orange.paint("TrueColor RGB (255, 110, 64)"));
    println!("  {}", deep_ocean.paint(" RGB Foreground + Background "));
    println!("  {}", ansi_256_code.paint("ANSI-256 code #141 (Mauve)"));

    println!("\n=== 5. Custom CLI Help Theme Configuration ===");
    // Define a custom palette for the CLI parser help screen
    let custom_theme = Styles::plain()
        .header(Style::new().bold().fg(Color::Gold).underline())
        .usage(Style::new().bold().fg(Color::BrightCyan))
        .literal(Style::new().bold().fg(Color::Lime))
        .placeholder(Style::new().italic().fg(Color::Silver))
        .error(Style::new().bold().bg(Color::BrightRed).fg(Color::White))
        .valid(Style::new().bold().fg(Color::BrightGreen))
        .warning(Style::new().bold().fg(Color::Orange));

    let app = Xarp::new("themed-tool")
        .version("1.0.0")
        .about("A tool showcasing custom help screen aesthetics")
        .styles(custom_theme)
        .arg(
            Arg::new("target")
                .help("Compilation target architecture")
                .value_name("ARCH")
                .required(true),
        )
        .arg(
            Arg::new("optimize")
                .short('O')
                .long("optimize")
                .help("Optimization level (0-3)")
                .value_name("LEVEL")
                .default_value("3"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Silence standard logs")
                .action(ArgAction::SetTrue),
        );

    // Render the custom-themed help layout directly
    app.print_help();
}
