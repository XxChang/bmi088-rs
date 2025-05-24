use tock_registers::register_bitfields;

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum AccRegisters {
    CHIP_ID = 0x00,
    STATUS = 0x03,

    X_LSB = 0x12,
    X_MSB = 0x13,
    Y_LSB = 0x14,
    Y_MSB = 0x15,
    Z_LSB = 0x16,
    Z_MSB = 0x17,

    SENSORTIME_0 = 0x18,
    TEMP_MSB = 0x22,
    CONF = 0x40,
    RANGE = 0x41,
    PWR_CONF = 0x7C,
    PWR_CTRL = 0x7D,
    SOFTRESET = 0x7E,
}

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum GyroRegisters {
    CHIP_ID = 0x00,

    RATE_X_LSB = 0x02,
    RATE_X_MSB = 0x03,
    RATE_Y_LSB = 0x04,
    RATE_Y_MSB = 0x05,
    RATE_Z_LSB = 0x06,
    RATE_Z_MSB = 0x07,

    BANDWIDTH  = 0x10,

    SOFTRESET = 0x14,
    GYRO_SELF_TEST = 0x3C,
}

pub mod acc {
    use super::*;

    register_bitfields! [
        u8,

        pub Conf [
            BWP OFFSET(4) NUMBITS(4) [
                OSR4    = 0b1000,
                OSR2    = 0b1001,
                Normal  = 0b1010,
            ],

            ODR OFFSET(0) NUMBITS(4) [
                Hz12_5 = 0b0101,
                Hz25   = 0b0110,
                Hz50   = 0b0111,
                Hz100  = 0b1000,
                Hz200  = 0b1001,
                Hz400  = 0b1010,
                Hz800  = 0b1011,
                Hz1600 = 0b1100,
            ],
        ],

        pub Status [
            DRDY OFFSET(7) NUMBITS(1) [],
        ],
    ];
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
