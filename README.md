# AS1115 driver

[![Crates.io](https://img.shields.io/crates/v/as1115)](https://crates.io/crates/as1115)
[![Crates.io](https://img.shields.io/crates/d/as1115)](https://crates.io/crates/nonik0)
[![docs.rs](https://img.shields.io/docsrs/as1115)](https://docs.rs/as1115/latest/nonik0/)

[![lint](https://github.com/nonik0/as1115/actions/workflows/lint.yml/badge.svg)](https://github.com/nonik0/as1115/actions/workflows/lint.yml)
[![build](https://github.com/nonik0/as1115/actions/workflows/build.yml/badge.svg)](https://github.com/nonik0/as1115/actions/workflows/build.yml)

Driver for [amg AS115 LED Driver IC](https://look.ams-osram.com/m/7ed04145f58f44e2/original/AS1115-DS000206.pdf). Many thanks for @blemasle's existing [AS1115 Arduino library](https://github.com/blemasle/arduino-as1115), which I used as a reference implementation.

## Features:
 * Single dependency on embedded-hal v1.0
 * Examples for Arduino Uno using avr-hal
 * TBD

## Install

To install this driver in your project add the following line to your `Cargo.toml`'s `dependencies` table:

```toml
as1115 = "0.1.0"
```
