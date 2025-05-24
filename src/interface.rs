//! I2C/SPI interfaces
//! Modeified to https://github.com/eldruin/bmi160-rs/blob/master/src/interface.rs

use embedded_hal::spi::Operation;
use embedded_hal_async::{i2c, spi::SpiDevice};

use crate::{private, Error};

/// I2C interface
#[derive(Debug)]
pub struct I2cInterface<I2C> {
    pub(crate) i2c: I2C,
    pub(crate) address: u8,
}

/// SPI interface
#[derive(Debug)]
pub struct SpiInterface<SPI> {
    pub(crate) spi: SPI,
    pub(crate) has_dummy_byte: bool,
}

/// Async Write data
///
/// Safety: Only can be implemented by internal object
///         due to Sealed trait
#[allow(async_fn_in_trait)]
pub trait AsyncWriteData: private::Sealed {
    /// Error type
    type Error;
    /// Write to an u8 register
    async fn write_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error>;
    /// Write data. The first element corresponds to the starting address.
    async fn write_data(&mut self, payload: &mut [u8]) -> Result<(), Self::Error>;
}

impl<I2C, E> AsyncWriteData for I2cInterface<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    type Error = Error<E>;

    async fn write_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [register, data];
        let addr = self.address;
        self.i2c.write(addr, &payload).await.map_err(Error::IOError)
    }

    async fn write_data(&mut self, payload: &mut [u8]) -> Result<(), Self::Error> {
        let addr = self.address;
        self.i2c.write(addr, payload).await.map_err(Error::IOError)
    }
}

impl<SPI, E> AsyncWriteData for SpiInterface<SPI>
where
    SPI: SpiDevice<u8, Error = E>,
{
    type Error = Error<E>;

    async fn write_register(&mut self, register: u8, data: u8) -> Result<(), Self::Error> {
        let payload: [u8; 2] = [register, data];
        self.spi.write(&payload).await.map_err(Error::IOError)
    }

    async fn write_data(&mut self, payload: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.write(payload).await.map_err(Error::IOError)
    }
}

/// Async Read data
///
/// Safety: Only can be implemented by internal object
///         due to Sealed trait
#[allow(async_fn_in_trait)]
pub trait AsyncReadData: private::Sealed {
    /// Error type
    type Error;
    /// Read from an u8 register
    async fn read_register(&mut self, register: u8) -> Result<u8, Self::Error>;
    /// Read data. The first element corresponds to the starting address.
    async fn read_data(&mut self, payload: &mut [u8]) -> Result<(), Self::Error>;
}

impl<I2C, E> AsyncReadData for I2cInterface<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    type Error = Error<E>;

    async fn read_register(&mut self, register: u8) -> Result<u8, Self::Error> {
        let mut data = [0];
        let addr = self.address;
        self.i2c
            .write_read(addr, &[register], &mut data)
            .await
            .map_err(Error::IOError)?;
        Ok(data[0])
    }

    async fn read_data(&mut self, payload: &mut [u8]) -> Result<(), Self::Error> {
        let len = payload.len();
        let addr = self.address;
        self.i2c
            .write_read(addr, &[payload[0]], &mut payload[1..len])
            .await
            .map_err(Error::IOError)
    }
}

impl<SPI, CommE> AsyncReadData for SpiInterface<SPI>
where
    SPI: SpiDevice<u8, Error = CommE>,
{
    type Error = Error<CommE>;

    async fn read_register(&mut self, register: u8) -> Result<u8, Self::Error> {
        if self.has_dummy_byte {
            let mut data = [0, 0];
            let address = [register | 0x80];
            // let transfer = Operation::Transfer(&mut data, &address);
            let write_address = Operation::Write(&address);
            let read_data = Operation::Read(&mut data);
            self.spi
                .transaction(&mut [write_address, read_data])
                .await
                .map_err(Error::IOError)?;
            Ok(data[1])
        } else {
            let mut data = [register | 0x80, 0];
            let operation = Operation::TransferInPlace(&mut data);
            self.spi
                .transaction(&mut [operation])
                .await
                .map_err(Error::IOError)?;
            Ok(data[1])
        }
    }

    async fn read_data(&mut self, payload: &mut [u8]) -> Result<(), Self::Error> {
        let operation = Operation::TransferInPlace(payload);
        self.spi
            .transaction(&mut [operation])
            .await
            .map_err(Error::IOError)?;
        Ok(())
    }
}
