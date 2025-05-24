use crate::{
    interface::{AsyncReadData, AsyncWriteData, I2cInterface, SpiInterface},
    register_address::{acc, AccRegisters},
    Bmi088, Error,
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(u8)]
pub enum AccelerometerRange {
    /// ±3g
    Scale3g = 0x00,
    /// ±6g
    #[default]
    Scale6g = 0x01,
    /// ±12g
    Scale12g = 0x02,
    /// ±24g
    Scale24g = 0x03,
}

impl AccelerometerRange {
    pub(crate) const fn multiplier(&self) -> f32 {
        match self {
            AccelerometerRange::Scale3g => {
                let power_2 = f32::from_bits(
                    ((1.0f32.to_bits() >> 23) + (1 as u32)) << 23
                );
                power_2 * 1.5 / 32768.0  
            },
            AccelerometerRange::Scale6g => {
                let power_2 = f32::from_bits(
                    ((1.0f32.to_bits() >> 23) + (2 as u32)) << 23
                );
                power_2 * 1.5 / 32768.0  
            },
            AccelerometerRange::Scale12g => {
                let power_2 = f32::from_bits(
                    ((1.0f32.to_bits() >> 23) + (3 as u32)) << 23
                );
                power_2 * 1.5 / 32768.0  
            },
            AccelerometerRange::Scale24g => {
                let power_2 = f32::from_bits(
                    ((1.0f32.to_bits() >> 23) + (4 as u32)) << 23
                );
                power_2 * 1.5 / 32768.0  
            },
        }
    }
}

pub struct Accelerometer<DI> {
    iface: DI,
    range: AccelerometerRange,
}

impl<SPI> Bmi088<SpiInterface<SPI>> {
    /// Create new instance of the BMI088 accelerometer communicating with SPI.
    ///
    /// Accelerometer will stay in I2C mode until it detects a rising edge
    /// on the CSB1 pin, so change the accelerometer to SPI mode in the
    /// initialization phase, the user could perform a dummy SPI read operation
    pub fn new_acc_with_spi(spi: SPI) -> Accelerometer<SpiInterface<SPI>> {
        Accelerometer {
            iface: SpiInterface {
                spi,
                has_dummy_byte: true,
            },
            range: Default::default(),
        }
    }
}

impl<I2C> Bmi088<I2cInterface<I2C>> {
    /// Create new instance of the BMI088 accelerometer communicating with I2C.
    pub fn new_acc_with_i2c(i2c: I2C, address: u8) -> Accelerometer<I2cInterface<I2C>> {
        Accelerometer {
            iface: I2cInterface { i2c, address },
            range: Default::default(),
        }
    }
}

impl<DI, E> Accelerometer<DI>
where
    DI: AsyncReadData<Error = Error<E>> + AsyncWriteData<Error = Error<E>>,
{
    pub async fn dummy_read(&mut self) -> Result<(), Error<E>> {
        self.iface.read_register(AccRegisters::CHIP_ID as _).await?;
        Ok(())
    }

    pub async fn soft_reset(&mut self) -> Result<(), Error<E>> {
        self.iface
            .write_register(AccRegisters::SOFTRESET as _, 0xB6)
            .await
    }

    pub async fn set_pwr_save(&mut self, mode: u8) -> Result<(), Error<E>> {
        self.iface
            .write_register(AccRegisters::PWR_CONF as _, mode)
            .await
    }

    pub async fn set_conf(&mut self, conf: u8) -> Result<(), Error<E>> {
        self.iface
            .write_register(AccRegisters::CONF as _, conf)
            .await
    }

    pub async fn set_range(&mut self, range: AccelerometerRange) -> Result<(), Error<E>> {
        self.range = range;
        self.iface
            .write_register(AccRegisters::RANGE as _, range as u8)
            .await
    }

    pub async fn enable_acc(&mut self) -> Result<(), Error<E>> {
        self.iface
            .write_register(AccRegisters::PWR_CTRL as _, 0x04)
            .await
    }

    /// Get chip ID
    pub async fn chip_id(&mut self) -> Result<u8, Error<E>> {
        self.iface.read_register(AccRegisters::CHIP_ID as _).await
    }

    pub async fn temperature(&mut self) -> Result<f32, Error<E>> {
        let mut data = [AccRegisters::TEMP_MSB as u8 | 0x80, 0, 0, 0];
        self.iface.read_data(&mut data).await?;
        let temperature = (((data[2] as i8) as i16) << 3) | (((data[3] as u16) >> 5) as i16);
        let temperature = temperature as f32 * 0.125 + 23.0;
        Ok(temperature)
    }

    pub async fn brust_read_xyz(&mut self) -> Result<(i16, i16, i16), Error<E>> {
        let status = self.iface.read_register(AccRegisters::STATUS as u8).await?;
        if !acc::Status::DRDY.is_set(status) {
            return Err(Error::NoDrdy);
        }
        let mut data = [AccRegisters::X_LSB as u8 | 0x80, 0, 0, 0, 0, 0, 0, 0];
        self.iface.read_data(&mut data).await?;
        let x_raw = i16::from_le_bytes([data[2], data[3]]);
        let y_raw = i16::from_le_bytes([data[4], data[5]]);
        let z_raw = i16::from_le_bytes([data[6], data[7]]);
        Ok((x_raw, y_raw, z_raw))
    }

    pub async fn xyz(&mut self) -> Result<(f32, f32, f32), Error<E>> {
        let (x, y, z) = self.brust_read_xyz().await?;
        let x = x as f32 * self.range.multiplier();
        let y = y as f32 * self.range.multiplier();
        let z = z as f32 * self.range.multiplier();
        Ok((x, y, z))
    }

    pub async fn sensor_time_us(&mut self) -> Result<u32, Error<E>> {
        let mut data = [AccRegisters::SENSORTIME_0 as u8 + 0x80, 0, 0, 0, 0];
        self.iface.read_data(&mut data).await?;
        let sensor_time = u32::from_le_bytes([data[2], data[3], data[4], 0x00]);
        let sensor_time = sensor_time * 39;
        Ok(sensor_time)
    }
}
