#![no_std]
#![no_main]

extern crate microbit;
extern crate panic_semihosting;

use core::fmt::Write;
use cortex_m_rt::entry;
use cortex_m_semihosting::hio;

use microbit::hal::prelude::*;
use microbit::hal::serial;
use microbit::hal::serial::BAUD115200;

#[entry]
fn main() -> ! {
    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Start").unwrap();

    if let Some(p) = microbit::Peripherals::take() {
        let gpio = p.GPIO.split();
        // Configure RX and TX pins accordingly
        let tx = gpio.pin24.into_push_pull_output().into();
        let rx = gpio.pin25.into_floating_input().into();
        // Configure serial communication
        let (mut tx, _) = serial::Serial::uart0(p.UART0, tx, rx, BAUD115200).split();

        let language = "Rust";
        let ranking = 1;
        write!(tx, "{} on embedded is #{}!\r\n", language, ranking).unwrap();
    }

    panic!("test-panic");
}
