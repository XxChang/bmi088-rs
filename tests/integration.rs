#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

#[defmt_test::tests]
mod tests {
    #[init]
    fn init() {
        let _ = embassy_stm32::init(Default::default());
    }

    #[test]
    fn test_convert_uint8_to_int16() {
        let temp_msb = 0xc1;
        let temp_lsb = 0x00;
        let temperature = (((temp_msb as i8) as i16) << 3) | (((temp_lsb as u16) >> 5) as i16);
        assert_eq!(temperature, -504);
    }
}
