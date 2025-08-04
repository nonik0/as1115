# AS1115 driver

[![Crates.io](https://img.shields.io/crates/v/as1115)](https://crates.io/crates/as1115)
[![Crates.io](https://img.shields.io/crates/d/as1115)](https://crates.io/crates/nonik0)
[![docs.rs](https://img.shields.io/docsrs/as1115)](https://docs.rs/as1115/latest/nonik0/)

[![lint](https://github.com/nonik0/as1115/actions/workflows/lint.yml/badge.svg)](https://github.com/nonik0/as1115/actions/workflows/lint.yml)
[![build](https://github.com/nonik0/as1115/actions/workflows/build.yml/badge.svg)](https://github.com/nonik0/as1115/actions/workflows/build.yml)

Driver for [amg AS115 LED Driver IC](https://look.ams-osram.com/m/7ed04145f58f44e2/original/AS1115-DS000206.pdf). Many thanks for @blemasle's existing [AS1115 Arduino library](https://github.com/blemasle/arduino-as1115), which I used as a reference implementation.

## Features:
 * Single dependency on embedded-hal v1.0
 * Examples for Arduino Uno using avr-hal (TODO)
 * TBD

## Install

To install this driver in your project add the following line to your `Cargo.toml`'s `dependencies` table:

```toml
as1115 = "0.1.0"
```


## How to Use

The AS1115 uses I2C for communication and requires access to an I2C bus. TODO about using embedded_hal trait for i2c

const NUM_DIGITS: u8 = 4; // AS1115 supports 1-8 seven-segment displays
let mut display: AS1115<_, NUM_DIGITS> = as1115::AS1115::new(HalI2CDevice);


Or to specify an address using the self-addressing feature (also needs pins wired correctly):

const NUM_DIGITS: u8 = 8;
const DIGITS_I2C_ADDR: u8 = 0x01;
let mut display: AS1115<_, NUM_DIGITS> = as1115::AS1115::new_with_addr(HalI2CDevice, DIGITS_I2C_ADDR);