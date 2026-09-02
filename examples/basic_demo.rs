//! Doc
use xarp::{Arg, ArgAction, Xarp};

/// doc comment
fn main() {
    let app = Xarp::new("mycli")
        .version("0.1.0")
        .about("A lightning-fast, styled CLI tool")
        // Positional argument
        .arg(
            Arg::new("input")
                .help("The input file to process")
                .value_name("FILE")
                .required(true),
        )
        // Flag
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose debugging output")
                .action(ArgAction::SetTrue),
        )
        // Valued Option
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Target path for generated files")
                .value_name("PATH")
                .default_value("dist/output.bin"),
        )
        // Subcommand
        .subcommand(
            Xarp::new("build")
                .about("Compile source packages into binary")
                .arg(
                    Arg::new("release")
                        .long("release")
                        .action(ArgAction::SetTrue),
                ),
        );

    let matches = app.get_matches();

    // Check Subcommand
    if let Some(("build", sub_matches)) = matches.subcommand() {
        let is_release = sub_matches.get_flag("release");
        println!("Building project (release mode: {is_release})");
        return;
    }

    // Read parsed values
    let input: String = matches.get_one("input").unwrap();
    let output: String = matches.get_one("output").unwrap();
    let is_verbose = matches.get_flag("verbose");

    println!("Input: {input}");
    println!("Output: {output}");
    println!("Verbose: {is_verbose}");
}
