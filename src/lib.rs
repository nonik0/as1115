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

    const fn const_pow(base: u32, exp: u32) -> u32 {
        let mut result = 1;
        let mut i = 0;
        while i < exp {
            result *= base;
            i += 1;
        }
        result
    }

    const fn max_unsigned_decimal() -> u32 {
        Self::const_pow(10, NUM_DIGITS as u32) - 1
    }

    const fn max_signed_decimal() -> i32 {
        if NUM_DIGITS == 1 {
            9 // No space for minus sign in 1-digit display
        } else {
            (Self::const_pow(10, (NUM_DIGITS - 1) as u32) - 1) as i32
        }
    }

    const fn max_unsigned_hex() -> u32 {
        Self::const_pow(16, NUM_DIGITS as u32) - 1
    }

    const fn max_signed_hex() -> i32 {
        if NUM_DIGITS == 1 {
            15 // No space for minus sign in 1-digit display
        } else {
            (Self::const_pow(16, (NUM_DIGITS - 1) as u32) - 1) as i32
        }
    }

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

    /// Display best-effort ASCII characters on the seven-segment display.
    /// Skips over any characters that do not have a valid segment mapping.
    /// Decimal points are included using the seven-segment display's DP segment.
    /// Truncates the input to fit NUM_DIGITS.
    pub fn display_ascii(&mut self, chars: &[u8]) -> Result<(), AS1115Error<E>> {
        let mut index = 0;
        let mut i = 0;

        while i < chars.len() && index < NUM_DIGITS {
            let c = chars[i];
            let mut segment_data = ascii_to_segment(c);

            if segment_data == 0 {
                i += 1;
                continue;
            }

            if i + 1 < chars.len() && chars[i + 1] == b'.' {
                segment_data |= segments::DP;
                i += 1;
            }

            self.set_digit_segment_data(index, segment_data)?;
            index += 1;
            i += 1;
        }
        Ok(())
    }

    /// Display an integer value in decimal format on the seven-segment display.
    /// Supports negative numbers by prepending a minus sign.
    /// Returns InvalidValue if the value is too large to fit in the display.
    pub fn display_value<T>(&mut self, value: T) -> Result<(), AS1115Error<E>>
    where
        T: ToPrimitive,
    {
        let signed_value = value.to_i32().ok_or(AS1115Error::InvalidValue)?;

        // Check if value will fit
        if signed_value >= 0 {
            if signed_value as u32 > Self::max_unsigned_decimal() {
                return Err(AS1115Error::InvalidValue);
            }
        } else {
            if -signed_value > Self::max_signed_decimal() {
                return Err(AS1115Error::InvalidValue);
            }
        }

        let is_negative = signed_value < 0;
        let mut num = signed_value.unsigned_abs();

        let mut digit_index = NUM_DIGITS;

        while digit_index > 0 && (num > 0 || digit_index == NUM_DIGITS) {
            digit_index -= 1;
            self.set_digit_segment_data(digit_index, NUMBERS[(num % 10) as usize])?;
            num /= 10;
        }

        if is_negative {
            digit_index -= 1;
            self.set_digit_segment_data(digit_index, MINUS_SIGN)?;
        }

        while digit_index > 0 {
            digit_index -= 1;
            self.set_digit_segment_data(digit_index, 0)?;
        }

        Ok(())
    }

    /// Display an integer value in hexadecimal format on the seven-segment display.
    /// Supports negative numbers by prepending a minus sign.
    /// Returns InvalidValue if the number is too large to fit in the display.
    pub fn display_hex_value<T>(&mut self, value: T) -> Result<(), AS1115Error<E>>
    where
        T: ToPrimitive,
    {
        let signed_value = value.to_i32().ok_or(AS1115Error::InvalidValue)?;

        // Check if value will fit
        if signed_value >= 0 {
            if signed_value as u32 > Self::max_unsigned_hex() {
                return Err(AS1115Error::InvalidValue);
            }
        } else {
            if -signed_value > Self::max_signed_hex() {
                return Err(AS1115Error::InvalidValue);
            }
        }

        let is_negative = signed_value < 0;
        let mut num = signed_value.unsigned_abs();

        let mut digit_index = NUM_DIGITS;

        while digit_index > 0 && (num > 0 || digit_index == NUM_DIGITS) {
            digit_index -= 1;
            self.set_digit_segment_data(digit_index, NUMBERS[(num % 16) as usize])?;
            num /= 16;
        }

        if is_negative {
            digit_index -= 1;
            self.set_digit_segment_data(digit_index, MINUS_SIGN)?;
        }

        while digit_index > 0 {
            digit_index -= 1;
            self.set_digit_segment_data(digit_index, 0)?;
        }

        Ok(())
    }

    /// Display a floating-point decimal value on the seven-segment display.
    /// Supports negative numbers by prepending a minus sign.
    /// Returns InvalidValue if the value won't fit with the given precision or if the precision value is invalid (0 or > NUM_DIGITS).
    #[cfg(feature = "display_float_value")]
    pub fn display_float_value<T>(&mut self, value: T, precision: u8) -> Result<(), AS1115Error<E>>
    where
        T: ToPrimitive,
    {
        let float_val = value.to_f32().ok_or(AS1115Error::InvalidValue)?;

        if precision < 1 || precision >= NUM_DIGITS || !float_val.is_finite() {
            return Err(AS1115Error::InvalidValue);
        }

        let is_negative = float_val.is_sign_negative();
        let abs_val = float_val.abs();

        // Check value bounds
        let integer_digits = if abs_val < 1.0 {
            1
        } else {
            let mut temp = abs_val as u32;
            let mut digits = 0;
            while temp > 0 {
                digits += 1;
                temp /= 10;
            }
            digits
        };
        let total_digits = (if is_negative { 1 } else { 0 }) + integer_digits + precision as u8;
        if total_digits > NUM_DIGITS {
            return Err(AS1115Error::InvalidValue);
        }

        // Scale number to integer value for formatting
        let mut digit_index = NUM_DIGITS;
        let mut scale_factor = 1.0f32;
        for _ in 0..precision {
            scale_factor *= 10.0;
        }
        let rounded_val = abs_val * scale_factor + 0.5;
        let mut digits = rounded_val as u32;

        for _ in 0..precision {
            digit_index -= 1;
            self.set_digit_segment_data(digit_index, NUMBERS[(digits % 10) as usize])?;
            digits /= 10;
        }

        if digits == 0 {
            digit_index -= 1;
            self.set_digit_segment_data(digit_index, NUMBERS[0] | segments::DP)?;
        } else {
            let mut first_digit = true;
            while digits > 0 {
                // possible for rounding to cause overflow
                if digit_index == 0 {
                    return Err(AS1115Error::InvalidValue);
                }

                digit_index -= 1;

                let mut segment_data = NUMBERS[digits as usize % 10];
                if first_digit {
                    segment_data |= segments::DP;
                    first_digit = false;
                }
                self.set_digit_segment_data(digit_index, segment_data)?;
                digits /= 10;
            }
        }

        if is_negative {
            digit_index -= 1;
            self.set_digit_segment_data(digit_index, MINUS_SIGN)?;
        }

        Ok(())
    }

    /// Display raw segment data on the seven-segment display.
    /// Truncates extra segment data beyond NUM_DIGITS.
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
    /// Returns InvalidLocation if the digit index is out of bounds.
    /// Returns InvalidValue if the character does not have a valid segment mapping.
    pub fn set_digit_ascii_char(&mut self, digit: u8, char: u8) -> Result<(), AS1115Error<E>> {
        if digit >= NUM_DIGITS {
            return Err(AS1115Error::InvalidLocation(digit));
        }

        let segments = ascii_to_segment(char);
        if segments == 0 {
            return Err(AS1115Error::InvalidValue);
        }

        self.set_digit_segment_data(digit, segments)
    }

    /// Set a specific digit to display a hexadecimal digit.
    /// Returns InvalidLocation if the digit index is out of bounds.
    /// Returns InvalidValue if the value is not a valid hexadecimal digit (0-15).
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
    /// Returns InvalidLocation if the digit index is out of bounds.
    pub fn set_digit_segment_data(
        &mut self,
        digit: u8,
        segment_data: u8,
    ) -> Result<(), AS1115Error<E>> {
        if digit >= NUM_DIGITS {
            return Err(AS1115Error::InvalidLocation(digit));
        }
        self.write_register(register::DIGIT_OFFSET + digit, segment_data)?;
        Ok(())
    }

    /// Set a specific digit to display a decimal digit (0-9).
    /// Returns InvalidLocation if the digit index is out of bounds.
    /// Returns InvalidValue if the value is not a valid decimal digit (0-9).
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
    /// Returns InvalidValue if the intensity value is out of range.
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
    /// Returns InvalidLocation if the digit index is out of bounds.
    /// Returns InvalidValue if the intensity value is out of range.
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
    /// Returns true if Rset is detected as open, false otherwise.
    pub fn rset_test_open(&mut self) -> Result<bool, AS1115Error<E>> {
        Ok((self.read_register(register::DISPLAY_TEST_MODE)?
            & register::display_test_mode::RSET_OPEN)
            != 0)
    }

    /// Tests whether external resistor Rset is shorted.
    /// Returns true if Rset is detected as shorted, false otherwise.
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

