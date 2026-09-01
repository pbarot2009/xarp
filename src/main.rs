pub mod color;
pub mod effect;
pub mod ramp;
pub mod style;

use ramp::Ramp;

fn main() {
    let cli = Ramp::new("ramp_test", "0.1.0-dev");
    cli.run();
}
