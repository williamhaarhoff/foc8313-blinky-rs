#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use embassy_stm32::gpio::{Level, Output, Speed};
//use embassy_time::Timer;
use embassy_stm32::time::Hertz;
use rtic::app;
use rtic_monotonics::systick::prelude::*;
use {defmt_rtt as _, panic_probe as _};
systick_monotonic!(Mono, 1_000);

pub mod pac {
    pub use embassy_stm32::pac::Interrupt as interrupt;
    pub use embassy_stm32::pac::*;
}

#[app(device = pac, peripherals = false, dispatchers = [SPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Initialize the systick interrupt & obtain the token to prove that we did
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
        Mono::start(cx.core.SYST, 72_000_000);

        defmt::info!("Hello World!");

        let mut led = Output::new(p.PC14, Level::High, Speed::Low);
        defmt::info!("high");
        led.set_high();

        // Schedule the blinking task
        blink::spawn(led).ok();

        (Shared {}, Local {})
    }

    #[task()]
    async fn blink(_cx: blink::Context, mut led: Output<'static>) {
        let mut state = true;
        loop {
            defmt::info!("blink");
            if state {
                led.set_high();
            } else {
                led.set_low();
            }
            state = !state;

            //Timer::after_secs(1).await;

            Mono::delay(1_000.millis()).await;
        }
    }
}
