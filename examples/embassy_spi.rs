#![no_std]
#![no_main]

use bmi088::Bmi088;
use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{gpio::{Level, Output, Speed}, mode::Async, spi::{Config, Spi}, time::Hertz};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Timer;

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use panic_probe as _;

type SpiBusAlias = Mutex<NoopRawMutex, Spi<'static, Async>>;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let spi = Spi::new(p.SPI2, p.PB13, p.PC3, p.PC2, p.DMA1_CH4, p.DMA1_CH3, spi_config);

    let spi_bus: SpiBusAlias = Mutex::new(spi);
    let gyro_cs = Output::new(p.PC14, Level::High, Speed::High);
    let acc_cs = Output::new(p.PC13, Level::High, Speed::High);

    let spi_acc = SpiDevice::new(&spi_bus, acc_cs);
    let spi_gyro =  SpiDevice::new(&spi_bus, gyro_cs);

    let mut gyro = Bmi088::new_gyro_with_spi(spi_gyro);
    let mut acc = Bmi088::new_acc_with_spi(spi_acc);

    Timer::after_millis(1).await;

    acc.dummy_read().await.unwrap();

    let chip_id = gyro.chip_id().await.unwrap();
    info!("gyro chip id: {:02X}", chip_id);

    let chip_id = acc.chip_id().await.unwrap();
    info!("acc chip id: {:02X}", chip_id);

    if let Err(_) = gyro.check_sensor().await {
        error!("Check gyro failed",);
    } else {
        info!("gyro function proper");
    }

    acc.enter_normal_mode().await.unwrap();

    Timer::after_micros(450).await;
    // let mut data = [0x00u8 + 0x80, 0];
    // let operation = Operation::TransferInPlace(&mut data);
    // spi_acc.transaction(&mut [operation]).await.unwrap();
    
    // let mut data = [0x00u8 + 0x80, 0];
    // let operation = Operation::TransferInPlace(&mut data);
    // spi_acc.transaction(&mut [operation]).await.unwrap();
    // info!("data {:02X}", data[1]);

    // let mut data = [0x00u8 + 0x80, 0];
    // let operation = Operation::TransferInPlace(&mut data);
    // spi_gyro.transaction(&mut [operation]).await.unwrap();
    // info!("data {:02X}", data[1]);

    loop {
        let (x, y, z) = gyro.data().await.unwrap();
        let temp = acc.temperature().await.unwrap();
        // let x = gyro.read_x_axis().await.unwrap();
        // let y = gyro.read_y_axis().await.unwrap();
        // let z = gyro.read_z_axis().await.unwrap();
        
        info!("x: {}, y: {}, z: {}, temp: {}", x, y, z, temp);
        // info!("Hello, World!");
        Timer::after_secs(1).await;
    }
}
