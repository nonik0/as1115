#![no_std]

mod constants;

pub use constants::*;
use embedded_hal::i2c::I2c;
use num_traits::ToPrimitive;

/// Convert an ASCII character to the corresponding seven-segment display encoding.
/// Supports only alphanumeric characters (0-9, a-z, A-Z).
pub fn ascii_to_segment(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => NUMBERS[(c - b'0') as usize],
        b'a'..=b'z' => LETTERS[(c - b'a') as usize],
        b'A'..=b'Z' => LETTERS[(c - b'A') as usize],
        _ => 0,
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AS1115Error<E> {
    I2cError(E),
    InvalidValue,
    InvalidLocation(u8),
}

impl<E> From<E> for AS1115Error<E> {
    fn from(error: E) -> Self {
        AS1115Error::I2cError(error)
    }
}


pub struct AS1115<I2C, const NUM_DIGITS: u8> {
    pub i2c: I2C,
    pub address: u8,
    intensity: [u8; MAX_DIGITS as usize], // ideally NUM_DIGITS
}

impl<I2C, E, const NUM_DIGITS: u8> AS1115<I2C, NUM_DIGITS>
where
    I2C: I2c<Error = E>,
{
    const NUM_DIGITS_VALID: () = {
        assert!(
            NUM_DIGITS >= 1 && NUM_DIGITS <= 8,
            "NUM_DIGITS must be between 1 and 8"
        );
    };

    /// Create a new AS1115 instance with the given I2C interface.
    pub fn new(i2c: I2C) -> Self {
        let _ = Self::NUM_DIGITS_VALID;
        Self {
            i2c,
            address: DEFAULT_ADDRESS,
            intensity: [0; MAX_DIGITS as usize],
        }
    }

    /// Create a new AS1115 instance with the given I2C interface and address (using self-addressing with KEYA, SEGF, and SEGG pins).
    pub fn new_with_addr(i2c: I2C, address: u8) -> Self {
        let _ = Self::NUM_DIGITS_VALID;
        Self {
            i2c,
            address,
            intensity: [0; MAX_DIGITS as usize],
        }
    }

    /// Destroy the AS1115 instance and return the underlying I2C interface.
    pub fn destroy(self) -> I2C {
        self.i2c
    }

    /// Initialize the AS1115 with the given global intensity.
    pub fn init(&mut self, intensity: u8) -> Result<(), AS1115Error<E>> {
        self.write_register_to_addr(
            DEFAULT_ADDRESS,
            register::SHUTDOWN_MODE,
            register::shutdown_mode::NORMAL_OPERATION | register::shutdown_mode::RESET_FEATURE,
        )?;

        if self.address != DEFAULT_ADDRESS {
            self.write_register_to_addr(
                DEFAULT_ADDRESS,
                register::SELF_ADDRESSING,
                register::self_addressing::USER_SET_ADDR,
            )?;
        }

        self.write_register(register::DECODE_MODE, register::decode_mode::NO_DECODE)?;
        self.write_register(register::SCAN_LIMIT, NUM_DIGITS - 1)?;
        self.set_intensity(intensity)?;

        Ok(())
    }

    /// Clear all digit data.
    pub fn clear(&mut self) -> Result<(), AS1115Error<E>> {
        for i in 0..NUM_DIGITS {
            self.set_digit_segment_data(i, 0)?;
        }
        Ok(())
    }

    /// Display janky ASCII characters on the seven-segment display.
    pub fn display_ascii(&mut self, chars: &[u8]) -> Result<(), AS1115Error<E>> {
        let mut index = 0;
        for c in chars {
            let segment_data = ascii_to_segment(*c);
            self.set_digit_segment_data(index, segment_data)?;
            index += 1;
            if index >= NUM_DIGITS {
                break;
            }
        }
        Ok(())
    }

    /// Display an integer value in decimal format on the seven-segment display.
    pub fn display_value<T>(&mut self, value: T) -> Result<(), AS1115Error<E>>
    where
        T: ToPrimitive,
    {
        let mut num = value.to_u32().ok_or(AS1115Error::InvalidValue)?;
        for i in 0..NUM_DIGITS {
            self.set_digit_segment_data(NUM_DIGITS - 1 - i, NUMBERS[(num % 10) as usize])?;
            num /= 10;
        }
        Ok(())
    }

    /// Display an integer value in hexadecimal format on the seven-segment display.
    pub fn display_hex_value<T>(&mut self, value: T) -> Result<(), AS1115Error<E>>
    where
        T: ToPrimitive,
    {
        let mut num = value.to_u32().ok_or(AS1115Error::InvalidValue)?;
        for i in 0..NUM_DIGITS {
            self.set_digit_segment_data(NUM_DIGITS - 1 - i, NUMBERS[(num % 16) as usize])?;
            num /= 16;
        }
        Ok(())
    }

    /// Display raw segment data on the seven-segment display.
    pub fn display_segments(&mut self, segments: &[u8]) -> Result<(), AS1115Error<E>> {
        for (index, &segment) in segments.iter().enumerate() {
            self.set_digit_segment_data(index as u8, segment)?;
            if index as u8 >= NUM_DIGITS {
                break;
            }
        }
        Ok(())
    }

    /// Read keyscan data from 16 keys.
    pub fn read_keys(&mut self) -> Result<u16, AS1115Error<E>> {
        let mut key_a = self.read_register(register::KEY_A)?;
        let key_b = self.read_register(register::KEY_B)?;

        // clear bits used for SEGG and SEGF pins on KEYA if self-addressing is enabled
        if self.address != DEFAULT_ADDRESS {
            key_a &= 0xFC;
        }

        Ok((key_a as u16) << 8 | (key_b as u16))
    }

    /// Set a specific digit to display an ASCII character.
    pub fn set_digit_ascii_char(&mut self, digit: u8, char: u8) -> Result<(), AS1115Error<E>> {
        if digit >= NUM_DIGITS {
            return Err(AS1115Error::InvalidLocation(digit));
        }
        let segments = ascii_to_segment(char);
        self.set_digit_segment_data(digit, segments)
    }

    /// Set a specific digit to display a hexadecimal digit.
    pub fn set_digit_hex_value(&mut self, digit: u8, value: u8) -> Result<(), AS1115Error<E>> {
        if digit >= NUM_DIGITS {
            return Err(AS1115Error::InvalidLocation(digit));
        }
        if value > 15 {
            return Err(AS1115Error::InvalidValue);
        }
        let segments = NUMBERS[value as usize];
        self.set_digit_segment_data(digit, segments)
    }

    /// Set a specific digit to display custom segment data.
    pub fn set_digit_segment_data(&mut self, digit: u8, segment_data: u8) -> Result<(), AS1115Error<E>> {
        if digit >= NUM_DIGITS {
            return Err(AS1115Error::InvalidLocation(digit));
        }
        self.write_register(register::DIGIT_OFFSET + digit, segment_data)?;
        Ok(())
    }

    /// Set a specific digit to display a decimal digit (0-9).
    pub fn set_digit_value(&mut self, digit: u8, value: u8) -> Result<(), AS1115Error<E>> {
        if digit >= NUM_DIGITS {
            return Err(AS1115Error::InvalidLocation(digit));
        }
        if value > 9 {
            return Err(AS1115Error::InvalidValue);
        }
        let segments = NUMBERS[value as usize];
        self.set_digit_segment_data(digit, segments)
    }

    /// Set the global intensity for all digits.
    pub fn set_intensity(&mut self, intensity: u8) -> Result<(), AS1115Error<E>> {
        if intensity > MAX_INTENSITY {
            return Err(AS1115Error::InvalidValue);
        }
        for i in 0..NUM_DIGITS {
            self.intensity[i as usize] = intensity;
        }
        self.write_register(register::GLOBAL_INTENSITY, intensity)?;
        Ok(())
    }

    /// Set the intensity for a specific digit.
    pub fn set_digit_intensity(&mut self, digit: u8, intensity: u8) -> Result<(), AS1115Error<E>> {
        if digit >= NUM_DIGITS {
            return Err(AS1115Error::InvalidLocation(digit));
        }
        if intensity > MAX_INTENSITY {
            return Err(AS1115Error::InvalidValue);
        }

        self.intensity[digit as usize] = intensity;

        let register = match digit {
            0..=1 => register::DIG01_INTENSITY,
            2..=3 => register::DIG23_INTENSITY,
            4..=5 => register::DIG45_INTENSITY,
            6..=7 => register::DIG67_INTENSITY,
            _ => return Err(AS1115Error::InvalidLocation(digit)),
        };

        // intensity register is read-only so we need a local cache to avoid overwriting paired digit's intensity
        let reg_value = if digit % 2 == 0 {
            self.intensity[(digit + 1) as usize] << 4 | self.intensity[digit as usize]
        } else {
            self.intensity[digit as usize] << 4 | self.intensity[(digit - 1) as usize]
        };

        self.write_register(register, reg_value)?;
        Ok(())
    }

    /// Enable or disable display test mode (all LED segments on).
    pub fn set_display_test(&mut self, enable: bool) -> Result<(), AS1115Error<E>> {
        let mut test_mode = self.read_register(register::DISPLAY_TEST_MODE)?;

        if enable {
            test_mode |= register::display_test_mode::DISP_TEST;
        } else {
            test_mode &= !register::display_test_mode::DISP_TEST;
        }

        self.write_register(register::DISPLAY_TEST_MODE, test_mode)?;
        Ok(())
    }

    /// Tests whether external resistor Rset is open.
    pub fn rset_test_open(&mut self) -> Result<bool, AS1115Error<E>> {
        Ok((self.read_register(register::DISPLAY_TEST_MODE)?
            & register::display_test_mode::RSET_OPEN)
            != 0)
    }

    /// Tests whether external resistor Rset is shorted.
    pub fn rset_test_short(&mut self) -> Result<bool, AS1115Error<E>> {
        Ok((self.read_register(register::DISPLAY_TEST_MODE)?
            & register::display_test_mode::RSET_SHORT)
            != 0)
    }

    fn read_register(&mut self, register: u8) -> Result<u8, AS1115Error<E>> {
        let mut buffer = [0; 1];
        self.i2c
            .write_read(self.address, &[register], &mut buffer)?;
        Ok(buffer[0])
    }

    fn write_register(&mut self, register: u8, value: u8) -> Result<(), AS1115Error<E>> {
        self.write_register_to_addr(self.address, register, value)?;
        Ok(())
    }

    fn write_register_to_addr(
        &mut self,
        address: u8,
        register: u8,
        value: u8,
    ) -> Result<(), AS1115Error<E>> {
        self.i2c.write(address, &[register, value])?;
        Ok(())
    }
}
