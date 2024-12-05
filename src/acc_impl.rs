use crate::{interface::{AsyncReadData, AsyncWriteData, I2cInterface, SpiInterface}, register_address::AccRegisters, Bmi088, Error};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[repr(u8)]
pub enum AccelerometerRange {
    /// ±3g
    #[default]
    Scale3g = 0x00,
    /// ±6g
    Scale6g = 0x01,
    /// ±12g
    Scale12g = 0x02,
    /// ±24g
    Scale24g = 0x03,
}


pub struct Accelerometer<DI> {
    iface: DI,
}

impl<SPI> Bmi088<SpiInterface<SPI>> 
{
    /// Create new instance of the BMI088 accelerometer communicating with SPI.
    /// 
    /// Accelerometer will stay in I2C mode until it detects a rising edge
    /// on the CSB1 pin, so change the accelerometer to SPI mode in the 
    /// initialization phase, the user could perform a dummy SPI read operation
    pub fn new_acc_with_spi(spi: SPI) ->  Accelerometer<SpiInterface<SPI>> {
        Accelerometer {
            iface: SpiInterface { spi },
        }
    }
}

impl<I2C> Bmi088<I2cInterface<I2C>> {
    /// Create new instance of the BMI088 accelerometer communicating with I2C.
    pub fn new_acc_with_i2c(i2c: I2C, address: u8) ->  Accelerometer<I2cInterface<I2C>> {
        Accelerometer {
            iface: I2cInterface { i2c, address },
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

    pub async fn enter_normal_mode(&mut self) -> Result<(), Error<E>> {
        self.iface.write_register(AccRegisters::PWR_CTRL as _, 0x04).await
    }

    /// Get chip ID
    pub async fn chip_id(&mut self) -> Result<u8, Error<E>> {
        self.iface.read_register(AccRegisters::CHIP_ID as _).await
    }


    pub async fn temperature(&mut self) -> Result<f32, Error<E>> {
        let mut data = [AccRegisters::TEMP_MSB as u8 + 0x80, 0, 0];
        self.iface.read_data(&mut data).await?;
        let temp_lsb = data[2] as u16;
        let temp_msb = data[1] as u16;
        let temp_lsb = (temp_lsb >> 5) as u8;
        let temp_msb = (temp_msb << 3) as u8;
        let temperature = i16::from_le_bytes([temp_lsb, temp_msb]);
        let temperature = temperature as f32 * 0.125 + 23.0;
        Ok(temperature)
    }
}
