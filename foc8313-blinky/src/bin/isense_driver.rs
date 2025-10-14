#![no_std]
#![no_main]
use defmt::*;
use drivers::isense::Isense;
use drivers::pwm::{CompareOC4, Phase, Pwm3};
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::time::{hz, khz, Hertz};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    interrupt,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1_2 => drivers::isense::InterruptHandler<ADC1>;
});

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
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

    let mut isense_driver = Isense::new(p.ADC1, p.PA3);

    let mut pwm_driver = Pwm3::new(p.TIM3, p.PA6, p.PA7, p.PB0, CompareOC4, khz(16));
    pwm_driver.enable(Phase::A);
    pwm_driver.enable(Phase::B);
    pwm_driver.enable(Phase::C);
    let duty = pwm_driver.get_max_duty() / 16;
    let off: u16 = 0;

    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::TIM3);
    }

    let sequence = [
        (duty, off, off),
        (duty, duty, off),
        (off, duty, off),
        (off, duty, duty),
        (off, off, duty),
        (duty, off, duty),
    ];

    loop {
        for (a, b, c) in sequence.iter() {
            pwm_driver.set_duty(Phase::A, *a);
            pwm_driver.set_duty(Phase::B, *b);
            pwm_driver.set_duty(Phase::C, *c);
            let result = isense_driver.convert().await;
            info!("measured: {:?}", result);
            Timer::after_millis(50).await;
        }
    }
}
