pub const DEFAULT_ADDRESS: u8 = 0x00;
pub const MAX_DIGITS: u8 = 8;
pub const MAX_INTENSITY: u8 = 15; // 4 bits
//pub const DOT_MASK: u8 = 0x80;
pub const NUMBERS: [u8; 16] = [
    0x7E, 0x30, 0x6D, 0x79, 0x33, 0x5B, 0x5F, 0x70, 0x7F, 0x7B, 0x77, 0x1F, 0x4E, 0x3D, 0x4F, 0x47,
];
pub const LETTERS: [u8; 26] = [
    0x77, 0x1F, 0x4E, 0x3D, 0x4F, 0x47, 0x5E, 0x37, 0x30, 0x3C, 0x2F, 0x0E, 0x54, 0x15, 0x1D, 0x67,
    0x73, 0x05, 0x5B, 0x0F, 0x3E, 0x1C, 0x2A, 0x49, 0x3B, 0x25,
];

#[allow(dead_code)]
pub mod register {
    pub const DIGIT_OFFSET: u8 = 0x01; // Digit0 - Digit7
    pub const DECODE_MODE: u8 = 0x09;
    pub const GLOBAL_INTENSITY: u8 = 0x0A;
    pub const SCAN_LIMIT: u8 = 0x0B;
    pub const SHUTDOWN_MODE: u8 = 0x0C;
    pub const SELF_ADDRESSING: u8 = 0x2D; // bit 5 is set
    pub const FEATURE: u8 = 0x0E;
    pub const DISPLAY_TEST_MODE: u8 = 0x0F;
    pub const DIG01_INTENSITY: u8 = 0x10;
    pub const DIG23_INTENSITY: u8 = 0x11;
    pub const DIG45_INTENSITY: u8 = 0x12;
    pub const DIG67_INTENSITY: u8 = 0x13;
    pub const DIAG_DIGIT_0: u8 = 0x14;
    pub const DIAG_DIGIT_1: u8 = 0x15;
    pub const DIAG_DIGIT_2: u8 = 0x16;
    pub const DIAG_DIGIT_3: u8 = 0x17;
    pub const DIAG_DIGIT_4: u8 = 0x18;
    pub const DIAG_DIGIT_5: u8 = 0x19;
    pub const DIAG_DIGIT_6: u8 = 0x1A;
    pub const DIAG_DIGIT_7: u8 = 0x1B;
    pub const KEY_A: u8 = 0x1C;
    pub const KEY_B: u8 = 0x1D;

    pub mod decode_mode {
        pub const NO_DECODE: u8 = 0x00; // no decode for digits 7:0
        pub const DECODE_ALL: u8 = 0xFF; // decode for digits 7:0
    }

    pub mod display_test_mode {
        pub const DISP_TEST: u8 = 0x01; // bit 0: optical display test
        pub const LED_SHORT: u8 = 0x02; // bit 1: starts a test for shorted LEDs
        pub const LED_OPEN: u8 = 0x04; // bit 2: starts a test for open LEDs
        pub const LED_TEST: u8 = 0x08; // bit 3: indicates an ongoing open/short LED test
        pub const LED_GLOBAL: u8 = 0x10; // bit 4: indicates that the last open/short LED test has detected an error
        pub const RSET_OPEN: u8 = 0x20; // bit 5: checks if external resistor Rset is open
        pub const RSET_SHORT: u8 = 0x40; // bit 6: checks if external resistor Rset is shorted
    }

    pub mod feature {
        pub const CLK_EN: u8 = 0x01; // bit 0: external clock active
        pub const REG_RESET: u8 = 0x02; // bit 1: resets all control registers except the feature register
        pub const DECODE_SET: u8 = 0x04; // bit 2: sets the display decoding for the selected digits (0: Code-B, 1: HEX)
        pub const BLINK_EN: u8 = 0x10; // bit 4: enables blinking
        pub const BLINK_FREQ_SET: u8 = 0x20; // bit 5: sets the blink with low frequency (with internal oscillator enabled)
        pub const SYNC: u8 = 0x40; // bit 6: synchronizes blinking on the rising edge of pin LD/CS
        pub const BLINK_START: u8 = 0x80; // bit 7: starts blinking with display phase enabled
    }

    // pub mod display_test_mode {
    //     pub const DISPLAY_TEST_MODE: u8 = 0x01;
    //     pub const NORMAL_OPERATION: u8 = 0x00;
    // }

    pub mod self_addressing {
        pub const FACTORY_SET_ADDR: u8 = 0x00;
        pub const USER_SET_ADDR: u8 = 0x01;
    }

    pub mod shutdown_mode {
        pub const SHUTDOWN_MODE: u8 = 0x00; // bit 0 clear: shutdown mode
        pub const NORMAL_OPERATION: u8 = 0x01; // bit 0 set: normal operation
        pub const RESET_FEATURE: u8 = 0x00; // bit 7 clear: reset feature register to default settings
        pub const PRESERVE_FEATURE: u8 = 0x80; // bit 7 set: feature register unchanged
    }

}
