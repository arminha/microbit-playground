#![no_std]
#![no_main]

extern crate microbit;
extern crate panic_semihosting;

use core::fmt::Write;
use cortex_m_rt::entry;

use microbit::hal::delay::Delay;
use microbit::hal::gpio::gpio::{PIN24, PIN25};
use microbit::hal::gpio::{Floating, Input};
use microbit::hal::prelude::*;
use microbit::hal::serial;
use microbit::hal::serial::{Tx, BAUD115200};
use microbit::led::Display;
use microbit::UART0;

#[entry]
fn main() -> ! {
    if let Some(p) = microbit::Peripherals::take() {
        let gpio = p.GPIO.split();
        let mut tx = init_uart(gpio.pin24, gpio.pin25, p.UART0);

        let language = "Rust";
        let ranking = 1;
        write!(tx, "{} on embedded is #{}!\r\n", language, ranking).unwrap();

        let mut delay = Delay::new(p.TIMER0);

        // Get row and column for display
        let row1 = gpio.pin13.into_push_pull_output();
        let row2 = gpio.pin14.into_push_pull_output();
        let row3 = gpio.pin15.into_push_pull_output();
        let col1 = gpio.pin4.into_push_pull_output();
        let col2 = gpio.pin5.into_push_pull_output();
        let col3 = gpio.pin6.into_push_pull_output();
        let col4 = gpio.pin7.into_push_pull_output();
        let col5 = gpio.pin8.into_push_pull_output();
        let col6 = gpio.pin9.into_push_pull_output();
        let col7 = gpio.pin10.into_push_pull_output();
        let col8 = gpio.pin11.into_push_pull_output();
        let col9 = gpio.pin12.into_push_pull_output();
        let mut leds = Display::new(
            col1, col2, col3, col4, col5, col6, col7, col8, col9, row1, row2, row3,
        );

        let heart = [
            [0, 1, 0, 1, 0],
            [1, 0, 1, 0, 1],
            [1, 0, 0, 0, 1],
            [0, 1, 0, 1, 0],
            [0, 0, 1, 0, 0],
        ];

        loop {
            leds.display(&mut delay, heart, 1000);
            leds.clear();
            delay.delay_ms(250_u32);
        }
    }
    panic!("End")
}

fn init_uart(
    pin24: PIN24<Input<Floating>>,
    pin25: PIN25<Input<Floating>>,
    uart: UART0,
) -> Tx<UART0> {
    // Configure RX and TX pins accordingly
    let tx = pin24.into_push_pull_output().into();
    let rx = pin25.into_floating_input().into();
    // Configure serial communication
    let (tx, _) = serial::Serial::uart0(uart, tx, rx, BAUD115200).split();
    tx
}
