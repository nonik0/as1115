extern crate as1115;

const NUM_DIGITS: u8 = 4;

struct MockI2c;

impl embedded_hal::i2c::ErrorType for MockI2c {
    type Error = embedded_hal::i2c::ErrorKind;
}

impl embedded_hal::i2c::I2c for MockI2c {
    fn write(&mut self, _address: u8, _data: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn read(&mut self, _address: u8, _buffer: &mut [u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write_read(
        &mut self,
        _address: u8,
        _write: &[u8],
        _read: &mut [u8],
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn decimal_value_test() {
    let mut display: as1115::AS1115<_, NUM_DIGITS> = as1115::AS1115::new(MockI2c);

    assert!(display.display_value(9999).is_ok());
    assert!(display.display_value(10000).is_err());
    assert!(display.display_value(-999).is_ok());
    assert!(display.display_value(-1000).is_err());
}

#[test]
fn hexadecimal_value_test() {
    let mut display: as1115::AS1115<_, NUM_DIGITS> = as1115::AS1115::new(MockI2c);

    assert!(display.display_hex_value(0xFFFF).is_ok());
    assert!(display.display_hex_value(0x10000).is_err());
    assert!(display.display_hex_value(-0xFFF).is_ok());
    assert!(display.display_hex_value(-0x1000).is_err());
}

#[cfg(feature = "display_float_value")]
#[test]
fn float_value_test() {
    let mut display: as1115::AS1115<_, NUM_DIGITS> = as1115::AS1115::new(MockI2c);

    // precision values
    assert!(display.display_float_value(0.0, 0).is_err());
    assert!(display.display_float_value(0.0, 1).is_ok());
    assert!(display.display_float_value(0.0, NUM_DIGITS - 1).is_ok());
    assert!(display.display_float_value(0.0, NUM_DIGITS).is_err());

    // integer values
    assert!(display.display_float_value(123.0, 1).is_ok());
    assert!(display.display_float_value(1234.0, 1).is_err());

    // test rounding overflow
    assert!(display.display_float_value(0.9999, 3).is_ok()); // rounds to 1.000
    assert!(display.display_float_value(9.9999, 3).is_err()); // rounds to 10.000
}
