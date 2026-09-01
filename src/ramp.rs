use std::env::{self};

pub struct Ramp {
    pub name: &'static str,
    pub version: &'static str,
}

impl Ramp {
    pub fn new(name: &'static str, version: &'static str) -> Self {
        Self { name, version }
    }

    pub fn run(&self) {
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
