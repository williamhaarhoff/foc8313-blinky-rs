#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _; // <- RTT logging
                    // use defmt_serial as _;
                    // use defmt_panic as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::Uart;
use embassy_time::{Duration, Timer};
use panic_probe as _;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("🔌 Hello from Embassy STM32!");
    // configure clocks for 72MHz, with external oscillator
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz::hz(16_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV2,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
        config.rcc.adc_pre = ADCPrescaler::DIV6;
    }
    let p = embassy_stm32::init(config);
    let mut led = Output::new(p.PC14, Level::Low, Speed::Low);

    loop {
        info!("🔆 LED on");
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        info!("🌑 LED off");
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
