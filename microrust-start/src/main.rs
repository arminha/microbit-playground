#![no_std]
#![no_main]

use panic_semihosting as _;

use core::fmt::Write;
use cortex_m_rt::entry;
use microbit::{
    display::blocking::Display,
    hal::{
        prelude::*,
        uart::{self, Baudrate, Parity},
        Timer,
    },
};

#[entry]
fn main() -> ! {
    if let Some(board) = microbit::Board::take() {
        let mut serial = uart::Uart::new(
            board.UART0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );

        let language = "Rust";
        let ranking = 1;
        write!(serial, "{} on embedded is #{}!\r\n", language, ranking).unwrap();

        let mut timer = Timer::new(board.TIMER0);
        let mut leds = Display::new(board.display_pins);

        let heart = [
            [0, 1, 0, 1, 0],
            [1, 0, 1, 0, 1],
            [1, 0, 0, 0, 1],
            [0, 1, 0, 1, 0],
            [0, 0, 1, 0, 0],
        ];

        loop {
            leds.show(&mut timer, heart, 1000);
            leds.clear();
            timer.delay_ms(250_u32);
        }
    }
    panic!("End")
}
