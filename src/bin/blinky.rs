#![no_std]
#![no_main]
// #![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _; // <- RTT logging
                    // use defmt_panic as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
// use embassy_stm32::time::Hertz;
use embassy_stm32::Peripherals;
use embassy_time::{Duration, Timer};
use panic_probe as _;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("🔌 Hello from Embassy STM32!");
    let p = embassy_stm32::init(Default::default());

    // Setup LED pin PA5 (change if using different board)
    let mut led = Output::new(p.PC14, Level::Low, Speed::Low);
    println!("cargo:rustc-link-arg=-Tlink.x");

    loop {
        info!("🔆 LED on");
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        info!("🌑 LED off");
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
