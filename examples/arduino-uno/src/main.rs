#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use as1115::AS1115;
use panic_halt as _;

const NUM_DIGITS: u8 = 3; // AS1115 support 1-8 seven-segment digits
const DEFAULT_INTENSITY: u8 = 3;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50_000,
    );

    let mut display: AS1115<_, NUM_DIGITS> = AS1115::new(i2c);
    display.init(DEFAULT_INTENSITY).unwrap();
    display.clear().unwrap();

    ufmt::uwriteln!(&mut serial, "Setting intensity...").unwrap_infallible();
    for intensity in 0..=as1115::MAX_INTENSITY {
        let digit = intensity % NUM_DIGITS;
        display.set_digit_intensity(digit, intensity).unwrap();
        display.set_digit_hex_value(digit, intensity).unwrap();
        arduino_hal::delay_ms(200);
    }
    display.set_intensity(DEFAULT_INTENSITY).unwrap();

    ufmt::uwriteln!(&mut serial, "Scrolling ASCII chars...").unwrap_infallible();
    let mut msg = [b' '; NUM_DIGITS as usize];
    for offset in 0..=(26 + NUM_DIGITS * 2) {
        for (i, m) in msg.iter_mut().enumerate() {
            let idx = offset as i8 + i as i8 - NUM_DIGITS as i8;
            *m = if (0..26).contains(&idx) {
                b'A' + idx as u8
            } else {
                b' '
            };
        }
        display.display_ascii(&msg).unwrap();
        arduino_hal::delay_ms(200);
    }

    ufmt::uwriteln!(&mut serial, "Counting up...").unwrap_infallible();
    let mut i: usize = 0;
    loop {
        display.display_value(i).unwrap();
        i += 1;

        arduino_hal::delay_ms(300);
    }
}
