#![no_main]
#![no_std]

use panic_halt as _;

use microbit::{
    display::nonblocking::GreyscaleImage,
    hal::{
        gpio::{Input, Pin, PullUp},
        prelude::InputPin,
    },
};
use rtic::app;

fn heart_image(inner_brightness: u8) -> GreyscaleImage {
    let b = inner_brightness;
    GreyscaleImage::new(&[
        [0, 7, 0, 7, 0],
        [7, b, 7, b, 7],
        [7, b, b, b, 7],
        [0, 7, b, 7, 0],
        [0, 0, 7, 0, 0],
    ])
}

fn author_image(step: u8) -> GreyscaleImage {
    let offset = step / 3;
    let slide: &[[u8; 8]; 5] = &[
        [0, 0, 7, 0, 0, 7, 0, 7],
        [0, 7, 0, 7, 0, 7, 0, 7],
        [0, 7, 7, 7, 0, 7, 7, 7],
        [0, 7, 0, 7, 0, 7, 0, 7],
        [0, 7, 0, 7, 0, 7, 0, 7],
    ];
    let image = &mut [[0u8; 5]; 5];
    for y in 0..5 {
        for x in 0..5 {
            image[y][x] = slide[y][(x + offset as usize) % 8];
        }
    }
    GreyscaleImage::new(image)
}

fn rust_image() -> GreyscaleImage {
    GreyscaleImage::new(&[
        [0, 7, 7, 0, 0],
        [0, 7, 0, 7, 0],
        [0, 7, 7, 0, 0],
        [0, 7, 0, 7, 0],
        [0, 7, 0, 7, 0],
    ])
}

#[derive(Clone, Copy, Debug)]
pub enum Images {
    Heart,
    Rust,
    Author,
}

impl Images {
    fn toggle(self) -> Images {
        match self {
            Images::Heart => Images::Rust,
            Images::Rust => Images::Author,
            Images::Author => Images::Heart,
        }
    }
}

pub struct Button {
    pin: Pin<Input<PullUp>>,
    was_pressed: bool,
}

impl Button {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Button {
            pin: pin.into_pullup_input(),
            was_pressed: false,
        }
    }
    /// Returns true if button is pressed
    fn is_pressed(&self) -> bool {
        self.pin.is_low().unwrap()
    }

    fn check_rising_edge(&mut self) -> bool {
        let mut rising_edge = false;

        let is_pressed = self.is_pressed();
        // Only trigger on "rising edge" of the signal
        // Term: "Edge Triggering"
        if self.was_pressed && !is_pressed {
            // Was pressed, now isn't:
            rising_edge = true;
        }
        self.was_pressed = is_pressed;
        rising_edge
    }
}

#[app(device = microbit::pac, peripherals = true)]
mod app {

    use microbit::{
        board::Board,
        display::nonblocking::{Display, Frame, MicrobitFrame},
        hal::{
            clocks::Clocks,
            rtc::{Rtc, RtcInterrupt},
        },
        pac,
    };
    use rtt_target::{rprintln, rtt_init_print};
    use super::*;

    #[shared]
    struct Shared {
        display: Display<pac::TIMER1>,
    }

    #[local]
    struct Local {
        anim_timer: Rtc<pac::RTC0>,
        button_a: Button,
        button_b: Button,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        let board = Board::new(cx.device, cx.core);

        // Starting the low-frequency clock (needed for RTC to work)
        Clocks::new(board.CLOCK).start_lfclk();

        // RTC at 16Hz (32_768 / (2047 + 1))
        // 16Hz; 62.5ms period
        let mut rtc0 = Rtc::new(board.RTC0, 2047).unwrap();
        rtc0.enable_event(RtcInterrupt::Tick);
        rtc0.enable_interrupt(RtcInterrupt::Tick, None);
        rtc0.enable_counter();

        let display = Display::new(board.TIMER1, board.display_pins);
        let button_a = Button::new(board.buttons.button_a.degrade());
        let button_b = Button::new(board.buttons.button_b.degrade());

        rprintln!("Init Complete");

        (
            Shared { display },
            Local {
                anim_timer: rtc0,
                button_a,
                button_b,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = TIMER1, priority = 2,
           shared = [display])]
    fn timer1(mut cx: timer1::Context) {
        cx.shared
        .display
        .lock(|display| display.handle_display_event());
    }

    #[task(binds = RTC0, priority = 1, shared = [display], local = [anim_timer, button_a, button_b,
        frame: MicrobitFrame = MicrobitFrame::default(),
        step: u8 = 0, animate: bool = true, images: Images = Images::Heart])]
    fn rtc0(cx: rtc0::Context) {
        let mut shared = cx.shared;
        let local = cx.local;

        local.anim_timer.reset_event(RtcInterrupt::Tick);

        if local.button_b.check_rising_edge() {
            let new_image = local.images.toggle();
            rprintln!("Showing {:?}", new_image);
            *local.images = new_image;
        }

        let mut animate = *local.animate;
        if local.button_a.check_rising_edge() {
            animate = !animate;
            if animate {
                rprintln!("Start animation");
            } else {
                rprintln!("Stop animation");
            }
            *local.step = 0;
            *local.animate = animate;
        }

        let inner_brightness = if animate {
            match *local.step {
                0..=8 => 9 - *local.step,
                9..=12 => 0,
                13..=20 => 21 - *local.step,
                21..=24 => 0,
                _ => unreachable!(),
            }
        } else {
            0
        };

        let image = match *local.images {
            Images::Heart => heart_image(inner_brightness),
            Images::Rust => rust_image(),
            Images::Author => author_image(*local.step),
        };
        local.frame.set(&image);
        shared.display.lock(|display| {
            display.show_frame(local.frame);
        });

        if animate {
            *local.step += 1;
            if *local.step == 25 {
                *local.step = 0
            };
        }
    }
}
