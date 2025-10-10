#![no_std]
#![no_main]
use drivers::pwm::{Pwm3, Phase, CompareOC4};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::time::{hz, Hertz};
use embassy_stm32::{
    gpio::{AfType, Flex, Level, Output, OutputType, Speed},
    interrupt, pac,
};
use embassy_stm32::Peri;
use {defmt_rtt as _, panic_probe as _};

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[interrupt]
fn TIM3() {
    unsafe {
        let pin = embassy_stm32::peripherals::PC14::steal();
        let mut pin = Flex::new(pin);
        pin.set_as_output(Speed::Low);
        pin.toggle();
    }
    pac::TIM3.sr().write(|w| w.set_uif(false));
    defmt::info!("interrupt! yay");
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("🔌 Hello from Embassy STM32!");
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

    let mut enable_pin = Output::new(p.PB1, Level::Low, Speed::Low);
    //let mut led = Output::new(p.PC14, Level::Low, Speed::Low);
    enable_pin.set_high();

    let mut pwm_driver = Pwm3::new(
        p.TIM3,
        p.PA6,
        p.PA7,
        p.PB0,
        CompareOC4,
        hz(16)
    );
    pwm_driver.enable(Phase::A);
    pwm_driver.enable(Phase::B);
    pwm_driver.enable(Phase::C);
    let duty = pwm_driver.get_max_duty() / 64;

    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::TIM3);
    }

    loop {
        pwm_driver.set_duty(Phase::A, duty);
    }
}

