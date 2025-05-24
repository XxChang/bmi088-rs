#![no_std]
#![no_main]

use bmi088::{acc_impl::AccelerometerRange, register_address, Bmi088};
use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    mode::Async,
    spi::{Config, Spi},
    time::Hertz,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Timer;

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use panic_probe as _;
use static_cell::StaticCell;

type SpiBusAlias = Mutex<NoopRawMutex, Spi<'static, Async>>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let spi = Spi::new(
        p.SPI2, p.PB13, p.PC3, p.PC2, p.DMA1_CH4, p.DMA1_CH3, spi_config,
    );

    let spi_bus: SpiBusAlias = Mutex::new(spi);
    let gyro_cs = Output::new(p.PC14, Level::High, Speed::High);
    let acc_cs = Output::new(p.PC13, Level::High, Speed::High);

    static SPI_BUS: StaticCell<SpiBusAlias> = StaticCell::new();
    let spi_bus = SPI_BUS.init(spi_bus);

    spawner.must_spawn(read_acc(spi_bus, acc_cs));
    spawner.must_spawn(read_gyro(spi_bus, gyro_cs));
    // let spi_acc = SpiDevice::new(&spi_bus, acc_cs);
    // let spi_gyro = SpiDevice::new(&spi_bus, gyro_cs);

    // let mut gyro = Bmi088::new_gyro_with_spi(spi_gyro);
    // let mut acc = Bmi088::new_acc_with_spi(spi_acc);

    // gyro.soft_reset().await.unwrap();
    // acc.soft_reset().await.unwrap();

    // Timer::after_millis(1).await;

    // acc.dummy_read().await.unwrap();

    // let chip_id = gyro.chip_id().await.unwrap();
    // info!("gyro chip id: {:02X}", chip_id);

    // let chip_id = acc.chip_id().await.unwrap();
    // info!("acc chip id: {:02X}", chip_id);

    // if let Err(_) = gyro.check_sensor().await {
    //     error!("Check gyro failed",);
    // } else {
    //     info!("gyro function proper");
    // }

    // acc.set_pwr_save(0x00).await.unwrap();
    // acc.enable_acc().await.unwrap();

    // acc.set_conf(
    //     register_address::acc::Conf::ODR::Hz1600.value
    //         | register_address::acc::Conf::BWP::OSR4.value,
    // )
    // .await
    // .unwrap();
    // acc.set_range(AccelerometerRange::Scale12g).await.unwrap();
    // gyro.set_bandwidth(0x81).await.unwrap();

    // Timer::after_micros(450).await;

    // loop {
    //     let (x, y, z) = gyro.data().await.unwrap();
    //     let temp = acc.temperature().await.unwrap();
    //     let time = acc.sensor_time_us().await.unwrap();
    //     let (acc_x, acc_y, acc_z) = acc.xyz().await.unwrap();

    //     info!("x: {}, y: {}, z: {}, temp: {}", x, y, z, temp);
    //     info!("acc_x: {} m/s^2, acc_y: {} m/s^2, acc_z: {} m/s^2", acc_x, acc_y, acc_z);
    //     info!("time: {} s", time / 1000000);
        // info!("Hello, World!");
    //     Timer::after_secs(1).await;
    // }
}

#[embassy_executor::task]
async fn read_acc(spi_bus: &'static SpiBusAlias, cs: Output<'static>) {
    let spi_acc = SpiDevice::new(&spi_bus, cs);
    let mut acc = Bmi088::new_acc_with_spi(spi_acc);

    acc.soft_reset().await.unwrap();

    Timer::after_millis(1).await;

    acc.dummy_read().await.unwrap();

    let chip_id = acc.chip_id().await.unwrap();
    info!("acc chip id: {:02X}", chip_id);

    acc.set_pwr_save(0x00).await.unwrap();
    acc.enable_acc().await.unwrap();

    acc.set_conf(
        register_address::acc::Conf::ODR::Hz1600.value
            | register_address::acc::Conf::BWP::OSR4.value,   
    )
    .await
    .unwrap();
    acc.set_range(AccelerometerRange::Scale12g).await.unwrap();
    
    Timer::after_micros(450).await;

    loop {
        if let Ok((acc_x, acc_y, acc_z)) = acc.xyz().await {
            let temp = acc.temperature().await.unwrap();
            let micro_seconds = acc.sensor_time_us().await.unwrap();
            info!("acc_x: {} m/s^2, acc_y: {} m/s^2, acc_z: {} m/s^2", acc_x, acc_y, acc_z);
            info!("temp: {} C", temp);
            info!("time: {} us", micro_seconds);
        }
        // Timer::after_micros(1).await;
    }
}

#[embassy_executor::task]
async fn read_gyro(spi_bus: &'static SpiBusAlias, cs: Output<'static>) {
    let spi_gyro = SpiDevice::new(&spi_bus, cs);
    let mut gyro = Bmi088::new_gyro_with_spi(spi_gyro);

    gyro.soft_reset().await.unwrap();

    Timer::after_micros(1).await;

    let chip_id = gyro.chip_id().await.unwrap();
    info!("gyro chip id: {:02X}", chip_id);

    if let Err(_) = gyro.check_sensor().await {
        error!("Check gyro failed",);
    } else {
        info!("gyro function proper");
    }

    gyro.set_bandwidth(0x81).await.unwrap();

    Timer::after_micros(450).await;

    // loop {
    //     let (x, y, z) = gyro.data().await.unwrap();
    //     info!("x: {}, y: {}, z: {}", x, y, z);
    //     Timer::after_micros(1).await;
    // }
}
