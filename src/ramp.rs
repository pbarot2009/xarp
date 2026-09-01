use crate::{
    color, effect,
    style::{self, Style, Styles},
};
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
            let err_style = Style::new().bold().fg(color::Color::BrightRed);
            println!("{err_style}error:{err_style:#} no arguments provided!\n");

            self.help();
        }
    }

    fn help(&self) {
        let name_style = Style::new()
            .bold()
            .bg(color::Color::Cyan)
            .fg(color::Color::Rgb(0, 0, 0));
        let version_style = Style::new().italic().underline().fg(color::Color::White);
        println!(
            "{name_style}{}{name_style:#}\t\t {version_style}{}{version_style:#}",
            self.name, self.version
        );
    }
}
