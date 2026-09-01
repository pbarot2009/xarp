mod style;
use std::env::{self};

struct Ramp {
    name: &'static str,
    version: &'static str,
}

impl Ramp {
    fn new(name: &'static str, version: &'static str) -> Self {
        Self { name, version }
    }

    fn run(&self) {
        let args = env::args();
        if args.len() < 2 {
            eprintln!("error: no arguments provided!");
            self.help();
        }
    }

    fn help(&self) {
        println!("{}\t\t{}", self.name, self.version);
    }
}

fn main() {
    let cli = Ramp::new("ramp_test", "0.1.0-dev");
    cli.run();
}
