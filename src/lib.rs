//! This is a platform agnostic Async Rust driver for the BMI088 IMU.
//! inertial measurement unit using the ['embedded-hal-async'] traits.
//! 
//! 

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use core::marker::PhantomData;

mod register_address;
pub mod interface;
pub mod gyro_impl;
pub mod acc_impl;

#[derive(Debug)]
pub struct Bmi088<DI> {
    _p: PhantomData<DI>,
}


#[derive(Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Error<E> {
    IOError(E),

    GyroFunctionUnproper,
}

mod private {
    use super::interface;
    pub trait Sealed {}

    impl<SPI> Sealed for interface::SpiInterface<SPI> {}
    impl<I2C> Sealed for interface::I2cInterface<I2C> {}
}