use tock_registers::register_bitfields;

// use crate::Error;

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum AccRegisters {
    CHIP_ID     = 0x00,
    PWR_CTRL    = 0x7D,
}

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum GyroRegisters {
    CHIP_ID         = 0x00,

    RATE_X_LSB      = 0x02,
    RATE_X_MSB      = 0x03,
    RATE_Y_LSB      = 0x04,
    RATE_Y_MSB      = 0x05,
    RATE_Z_LSB      = 0x06,
    RATE_Z_MSB      = 0x07,

    GYRO_SELF_TEST  = 0x3C,
}

register_bitfields! [
    u8,

    AccErr [
        ERROR OFFSET(2) NUMBITS(2) [
            NoError = 0b00,
            Error = 0b01
        ],

        FATAL_ERR OFFSET(0) NUMBITS(1) [],
    ],

    AccStatus [
        DRDY OFFSET(7) NUMBITS(1) [],
    ],

    TempLsb [
        TEMP OFFSET(5) NUMBITS(2) [],
    ],

    pub GyroSelfTest [
        OK   OFFSET(4) NUMBITS(1) [],
        FAIL OFFSET(2) NUMBITS(1) [],
        RDY  OFFSET(1) NUMBITS(1) [],
        TRIG OFFSET(0) NUMBITS(1) [],
    ]
];
