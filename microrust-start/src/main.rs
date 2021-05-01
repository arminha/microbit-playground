#![no_std]
#![no_main]

use panic_semihosting as _;

use core::fmt::Write;
use cortex_m_rt::entry;
use microbit::{
    display_pins,
    hal::{gpio::p0::Parts as P0Parts, prelude::*, uart::Baudrate, Timer},
    led::Display,
};

#[entry]
fn main() -> ! {
    if let Some(p) = microbit::pac::Peripherals::take() {
        let gpio = P0Parts::new(p.GPIO);

        let mut serial = microbit::serial_port!(gpio, p.UART0, Baudrate::BAUD115200);

        let language = "Rust";
        let ranking = 1;
        write!(serial, "{} on embedded is #{}!\r\n", language, ranking).unwrap();

        let mut timer = Timer::new(p.TIMER0);
        let pins = display_pins!(gpio);
        let mut leds = Display::new(pins);

        let heart = [
            [0, 1, 0, 1, 0],
            [1, 0, 1, 0, 1],
            [1, 0, 0, 0, 1],
            [0, 1, 0, 1, 0],
            [0, 0, 1, 0, 0],
        ];

        loop {
            leds.display(&mut timer, heart, 1000);
            leds.clear();
            timer.delay_ms(250_u32);
        }
    }
    panic!("End")
}
