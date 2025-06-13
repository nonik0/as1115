#![no_std]

mod constants;

pub use constants::*;
use embedded_hal::i2c::I2c;
use num_traits::ToPrimitive;

pub struct AS1115<I2C, const NUM_DIGITS: u8> {
    pub i2c: I2C,
    pub address: u8,
    intensity: [u8; MAX_DIGITS as usize], // ideally NUM_DIGITS
}

impl<I2C, E, const NUM_DIGITS: u8> AS1115<I2C, NUM_DIGITS>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            intensity: [0; MAX_DIGITS as usize],
        }
    }

    pub fn destroy(self) -> I2C {
        self.i2c
    }

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

    pub fn clear(&mut self) -> Result<(), AS1115Error<E>> {
        for i in 0..NUM_DIGITS {
            self.set_digit_data(i, 0)?;
        }
        Ok(())
    }

    pub fn display_ascii(&mut self, bytes: &[u8]) -> Result<(), AS1115Error<E>> {
        let mut index = 0;
        for c in bytes {
            let segment_data = match c {
                b'0'..=b'9' => NUMBERS[(c - b'0') as usize],
                b'a'..=b'z' => LETTERS[(c - b'a') as usize],
                b'A'..=b'Z' => LETTERS[(c - b'A') as usize],
                _ => 0,
            };
            self.set_digit_data(index, segment_data)?;
            index += 1;
            if index >= NUM_DIGITS {
                break;
            }
        }
        Ok(())
    }

    pub fn display_string(&mut self, string: &str) -> Result<(), AS1115Error<E>> {
        let mut index = 0;
        for c in string.chars() {
            let segment_data = match c {
                '0'..='9' => NUMBERS[(c as u8 - b'0') as usize],
                'a'..='z' => LETTERS[(c as u8 - b'a') as usize],
                'A'..='Z' => LETTERS[(c as u8 - b'A') as usize],
                _ => 0,
            };
            self.set_digit_data(index, segment_data)?;
            index += 1;
            if index >= NUM_DIGITS {
                break;
            }
        }
        Ok(())
    }

    pub fn display_number<T>(&mut self, number: T) -> Result<(), AS1115Error<E>>
    where
        T: ToPrimitive,
    {
        let mut num = number.to_u32().ok_or(AS1115Error::InvalidValue)?;
        for i in 0..NUM_DIGITS {
            let digit = num % 10;
            self.set_digit_data(NUM_DIGITS - 1 - i, NUMBERS[digit as usize])?;
            num /= 10;
        }
        Ok(())
    }

    pub fn display_hex_number<T>(&mut self, number: T) -> Result<(), AS1115Error<E>>
    where
        T: ToPrimitive,
    {
        let mut num = number.to_u32().ok_or(AS1115Error::InvalidValue)?;
        for i in 0..NUM_DIGITS {
            let digit = num % 16;
            let value = if digit < 10 {
                NUMBERS[digit as usize]
            } else {
                LETTERS[(digit - 10) as usize]
            };
            self.set_digit_data(NUM_DIGITS - 1 - i, value)?;
            num /= 16;
        }
        Ok(())
    }

    pub fn read_keys(&mut self) -> Result<u16, AS1115Error<E>> {
        let mut key_a = self.read_register(register::KEY_A)?;
        let key_b = self.read_register(register::KEY_B)?;

        // clear bits used for SEGG and SEGF pins on KEYA if self-addressing is enabled
        if self.address != DEFAULT_ADDRESS {
            key_a &= 0xFC;
        }

        Ok((key_a as u16) << 8 | (key_b as u16))
    }

    pub fn set_digit_data(&mut self, digit: u8, value: u8) -> Result<(), AS1115Error<E>> {
        if digit >= NUM_DIGITS {
            return Err(AS1115Error::InvalidLocation(digit));
        }
        self.write_register(register::DIGIT_OFFSET + digit, value)?;
        Ok(())
    }

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

    pub fn set_intensity_for_digit(
        &mut self,
        digit: u8,
        intensity: u8,
    ) -> Result<(), AS1115Error<E>> {
        let num_digits = if NUM_DIGITS > MAX_DIGITS {
            MAX_DIGITS
        } else {
            NUM_DIGITS
        };
        if digit >= num_digits {
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

    pub fn rset_test_open(&mut self) -> Result<bool, AS1115Error<E>> {
        Ok((self.read_register(register::DISPLAY_TEST_MODE)?
            & register::display_test_mode::RSET_OPEN)
            != 0)
    }

    pub fn rset_test_short(&mut self) -> Result<bool, AS1115Error<E>> {
        Ok((self.read_register(register::DISPLAY_TEST_MODE)?
            & register::display_test_mode::RSET_SHORT)
            != 0)
    }

    pub fn visual_test(&mut self, stop: bool) -> Result<(), AS1115Error<E>> {
        let mut test_mode = self.read_register(register::DISPLAY_TEST_MODE)?;

        if !stop {
            test_mode &= !register::display_test_mode::DISP_TEST;
        } else {
            test_mode |= register::display_test_mode::DISP_TEST;
        }

        self.write_register(register::DISPLAY_TEST_MODE, test_mode)?;
        Ok(())
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
