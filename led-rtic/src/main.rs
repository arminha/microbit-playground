#![no_main]
#![no_std]

use panic_halt as _;

use microbit::{
    display::{self, image::GreyscaleImage, Display, Frame, MicrobitDisplayTimer, MicrobitFrame},
    display_pins,
    gpio::DisplayPins,
    hal::{
        gpio::{p0::Parts as P0Parts, Input, Pin, PullUp},
        prelude::InputPin,
        rtc::{Rtc, RtcInterrupt},
    },
    pac,
};
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};

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
const APP: () = {
    struct Resources {
        display_pins: DisplayPins,
        display_timer: MicrobitDisplayTimer<pac::TIMER1>,
        anim_timer: Rtc<pac::RTC0>,
        display: Display<MicrobitFrame>,
        button_a: Button,
        button_b: Button,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        let p: pac::Peripherals = cx.device;

        // Starting the low-frequency clock (needed for RTC to work)
        p.CLOCK.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
        while p.CLOCK.events_lfclkstarted.read().bits() == 0 {}
        p.CLOCK.events_lfclkstarted.reset();

        // RTC at 16Hz (32_768 / (2047 + 1))
        // 16Hz; 62.5ms period
        let mut rtc0 = Rtc::new(p.RTC0, 2047).unwrap();
        rtc0.enable_event(RtcInterrupt::Tick);
        rtc0.enable_interrupt(RtcInterrupt::Tick, None);
        rtc0.enable_counter();

        let mut timer = MicrobitDisplayTimer::new(p.TIMER1);

        let p0parts = P0Parts::new(p.GPIO);
        let mut pins = display_pins!(p0parts);
        let button_a = Button::new(p0parts.p0_17.degrade());
        let button_b = Button::new(p0parts.p0_26.degrade());

        display::initialise_display(&mut timer, &mut pins);

        rprintln!("Init Complete");

        init::LateResources {
            display_pins: pins,
            display_timer: timer,
            anim_timer: rtc0,
            display: Display::new(),
            button_a,
            button_b,
        }
    }

    #[task(binds = TIMER1, priority = 2,
           resources = [display_timer, display_pins, display])]
    fn timer1(mut cx: timer1::Context) {
        display::handle_display_event(
            &mut cx.resources.display,
            cx.resources.display_timer,
            cx.resources.display_pins,
        );
    }

    #[task(binds = RTC0, priority = 1,
           resources = [anim_timer, display, button_a, button_b])]
    fn rtc0(mut cx: rtc0::Context) {
        static mut FRAME: MicrobitFrame = MicrobitFrame::const_default();
        static mut STEP: u8 = 0;
        static mut ANIMATE: bool = true;
        static mut IMAGES: Images = Images::Heart;

        cx.resources.anim_timer.reset_event(RtcInterrupt::Tick);

        if cx.resources.button_b.check_rising_edge() {
            let new_image = IMAGES.toggle();
            rprintln!("Showing {:?}", new_image);
            *IMAGES = new_image;
        }

        let mut animate = *ANIMATE;
        if cx.resources.button_a.check_rising_edge() {
            animate = !animate;
            if animate {
                rprintln!("Start animation");
            } else {
                rprintln!("Stop animation");
            }
            *STEP = 0;
            *ANIMATE = animate;
        }

        let inner_brightness = if animate {
            match *STEP {
                0..=8 => 9 - *STEP,
                9..=12 => 0,
                13..=20 => 21 - *STEP,
                21..=24 => 0,
                _ => unreachable!(),
            }
        } else {
            0
        };

        let image = match *IMAGES {
            Images::Heart => heart_image(inner_brightness),
            Images::Rust => rust_image(),
            Images::Author => author_image(*STEP),
        };
        FRAME.set(&image);
        cx.resources.display.lock(|display| {
            display.set_frame(FRAME);
        });

        if animate {
            *STEP += 1;
            if *STEP == 25 {
                *STEP = 0
            };
        }
    }
};
