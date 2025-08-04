# AS1115 driver

[![Crates.io](https://img.shields.io/crates/v/as1115)](https://crates.io/crates/as1115)
[![Crates.io](https://img.shields.io/crates/d/as1115)](https://crates.io/crates/as1115)
[![docs.rs](https://img.shields.io/docsrs/as1115)](https://docs.rs/as1115/latest/as1115/)

[![lint](https://github.com/nonik0/as1115/actions/workflows/lint.yml/badge.svg)](https://github.com/nonik0/as1115/actions/workflows/lint.yml)
[![build](https://github.com/nonik0/as1115/actions/workflows/build.yml/badge.svg)](https://github.com/nonik0/as1115/actions/workflows/build.yml)

Driver for [ams AS1115 LED Driver IC](https://look.ams-osram.com/m/7ed04145f58f44e2/original/AS1115-DS000206.pdf). Many thanks to @blemasle's existing [AS1115 Arduino library](https://github.com/blemasle/arduino-as1115), which I used as a reference implementation.

## Features:
 * Using embedded-hal v1.0 traits for maximum compatibility with embedded platforms
 * Generic numeric functions using num-traits for displaying decimal, hexadecimal and floating-point values
 * Support for displaying ASCII characters and custom segment data
 * Also supports hardware's global and individual brightness comtrol, self-test functionality, and keyscan input
 * Example for [Arduino Uno](examples/arduino-uno/), based on [avr-hal](https://github.com/Rahix/avr-hal/)

## Install

To install this driver in your project add the following line to your `Cargo.toml`'s `dependencies` table:

```toml
as1115 = "0.1.0"
```

For projects needing float support (increases binary size, avoid using on MCUs without native float support):

```toml
as1115 = { version = "0.1.0", features = ["display_float_value"] }
```

## How to Use

The AS1115 uses I2C for communication and requires access to an I2C bus that implements the `embedded_hal::i2c::I2c` trait. This allows the driver to work with any HAL that provides I2C functionality.

```rust
use as1115::AS1115;

const NUM_DIGITS: u8 = 4; // AS1115 supports 1-8 seven-segment displays
const INTENSITY: u8 = 3; // global brightness [0-15]

let mut as1115: AS1115<_, NUM_DIGITS> = AS1115::new(i2c_device);
as1115.init(INTENSITY).unwrap();

as1115.display_value(1234).unwrap();
as1115.display_hex_value(0xDEAD).unwrap();
as1115.display_ascii(b"HI").unwrap();

// Requires "display_float_value" feature
as1115.display_float_value(12.34, 2).unwrap();
```

Or to specify an address using the self-addressing feature (also needs pins wired correctly):

```rust
use as1115::AS1115;

const NUM_DIGITS: u8 = 4;
const INTENSITY: u8 = 2;
const I2C_ADDR: u8 = 0x01;

let mut as1115: AS1115<_, NUM_DIGITS> = AS1115::new_with_addr(i2c_device, I2C_ADDR);
as1115.init(INTENSITY).unwrap();

as1115.display_value(4321).unwrap();
as1115.display_hex_value(0xBEEF).unwrap();

// Requires "display_float_value" feature
as1115.display_float_value(432.1, 1).unwrap();
```


## TODO
- [ ] More display configuration options, e.g. enabling leading zeros for values, showing float special values, etc.
- [ ] Handle edge cases better for NUM_DIGITS=1
- [ ] More complete set of tests
- [ ] Implement rest of hardware self-testing functionality
