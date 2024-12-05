use crate::{interface::{AsyncReadData, AsyncWriteData, I2cInterface, SpiInterface}, register_address::{GyroRegisters, GyroSelfTest}, Bmi088, Error};


#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(u8)]
pub enum GyroscopeRange {
    /// 16.384 LSB/°/s <-> 61.0 m°/s/LSB
    #[default]
    Scale2000 = 0x00,
    /// 32.768 LSB/°/s <-> 30.5 m°/s/LSB
    Scale1000 = 0x01,
    /// 65.536 LSB/°/s <-> 15.3 m°/s/LSB
    Scale500 = 0x02,
    /// 131.072 LSB/°/s <-> 7.6 m°/s/LSB
    Scale250 = 0x03,
    /// 262.144 LSB/°/s <-> 3.8 m°/s/LSB
    Scale125 = 0x04,
}

impl GyroscopeRange {
    pub(crate) fn multiplier(&self) -> f32 {
        match self {
            GyroscopeRange::Scale2000 => 0.061,
            GyroscopeRange::Scale1000 => 0.0305,
            GyroscopeRange::Scale500 => 0.0153,
            GyroscopeRange::Scale250 => 0.0076,
            GyroscopeRange::Scale125 => 0.0038,
        }
    }
}

pub struct Gyroscope<DI> {
    iface: DI,
    gyro_range: GyroscopeRange,
}

impl<SPI> Bmi088<SpiInterface<SPI>> {
    /// Create new instance of the BMI088 accelerometer communicating with SPI.
    pub fn new_gyro_with_spi(spi: SPI) ->  Gyroscope<SpiInterface<SPI>> {
        Gyroscope {
            iface: SpiInterface { spi },
            gyro_range: Default::default(),
        }
    }
}

impl<I2C> Bmi088<I2cInterface<I2C>> {
    /// Create new instance of the BMI088 accelerometer communicating with I2C.
    pub fn new_gyro_with_i2c(i2c: I2C, address: u8) ->  Gyroscope<I2cInterface<I2C>> {
        Gyroscope {
            iface: I2cInterface { i2c, address },
            gyro_range: Default::default(),
        }
    }
}

impl<DI, E> Gyroscope<DI>
where 
    DI: AsyncReadData<Error = Error<E>> + AsyncWriteData<Error = Error<E>>,
{
    /// Get chip ID
    pub async fn chip_id(&mut self) -> Result<u8, Error<E>> {
        self.iface.read_register(GyroRegisters::CHIP_ID as _).await
    }

    pub async fn check_sensor(&mut self) -> Result<(), Error<E>> {
        let b = self.iface.read_register(GyroRegisters::GYRO_SELF_TEST as _).await?;

        let r = GyroSelfTest::OK;
        if !r.is_set(b) {
            Err(Error::GyroFunctionUnproper)
        } else {
            Ok(())
        }
    }

    pub async fn read_x_axis(&mut self) -> Result<i16, Error<E>> {
        let x_lsb = self.iface.read_register(GyroRegisters::RATE_X_LSB as _).await?;
        let x_msb = self.iface.read_register(GyroRegisters::RATE_X_MSB as _).await?;
        let x_raw = i16::from_le_bytes([x_lsb, x_msb]);
        Ok(x_raw)
    }

    pub async fn read_y_axis(&mut self) -> Result<i16, Error<E>> {
        let y_lsb = self.iface.read_register(GyroRegisters::RATE_Y_LSB as _).await?;
        let y_msb = self.iface.read_register(GyroRegisters::RATE_Y_MSB as _).await?;
        let y_raw = i16::from_le_bytes([y_lsb, y_msb]);
        Ok(y_raw)
    }

    pub async fn read_z_axis(&mut self) -> Result<i16, Error<E>> {
        let z_lsb = self.iface.read_register(GyroRegisters::RATE_Z_LSB as _).await?;
        let z_msb = self.iface.read_register(GyroRegisters::RATE_Z_MSB as _).await?;
        let z_raw = i16::from_le_bytes([z_lsb, z_msb]);
        Ok(z_raw)
    }

    pub async fn burst_read_xyz_rate(&mut self) -> Result<(i16, i16, i16), Error<E>> {
        let mut data = [GyroRegisters::RATE_X_LSB as u8 + 0x80, 0, 0, 0, 0, 0, 0];
        self.iface.read_data(&mut data).await?;
        let x_raw = i16::from_le_bytes([data[1], data[2]]);
        let y_raw = i16::from_le_bytes([data[3], data[4]]);
        let z_raw = i16::from_le_bytes([data[5], data[6]]);
        Ok((x_raw, y_raw, z_raw))
    }

    pub async fn data(&mut self) -> Result<(f32, f32, f32), Error<E>> {
        let (x, y, z) = self.burst_read_xyz_rate().await?;
        let x = x as f32 * self.gyro_range.multiplier();
        let y = y as f32 * self.gyro_range.multiplier();
        let z = z as f32 * self.gyro_range.multiplier();
        Ok((x, y, z))
    }
}