#![no_std]
#![no_main]

extern crate microbit;
extern crate panic_semihosting;

use cortex_m_rt::entry;
use cortex_m_semihosting::hio;
use core::fmt::Write;

#[entry]
fn main() -> ! {
    let mut stdout = hio::hstdout().unwrap();
    let language = "Rust";
    let ranking = 1;
    writeln!(stdout, "{} on embedded is #{}!", language, ranking).unwrap();
    panic!("test-panic");
}
