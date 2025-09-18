#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _; // <- RTT logging
                    // use defmt_serial as _;
                    // use defmt_panic as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::pac;
use embassy_stm32::pac::gpio::vals;
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

    let rcc = pac::RCC;
    let gpioc = pac::GPIOC;

    // Enable GPIOC peripheral clock
    rcc.apb2enr().modify(|w| w.set_gpiocen(true));

    // Configure PC14 as Output
    gpioc.cr(1).modify(|w| {
        use pac::gpio::vals::{CnfOut, Mode};
        w.set_mode(6, Mode::OUTPUT2MHZ);
        w.set_cnf_out(6, CnfOut::PUSH_PULL);
    });

    loop {
        info!("🔆 LED on");
        gpioc.odr().modify(|w| w.set_odr(14, vals::Odr::HIGH));
        Timer::after(Duration::from_millis(500)).await;

        info!("🌑 LED off");
        gpioc.odr().modify(|w| w.set_odr(14, vals::Odr::LOW));
        Timer::after(Duration::from_millis(500)).await;
    }
}
