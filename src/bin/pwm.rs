#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _; // <- RTT logging
                    // use defmt_serial as _;
                    // use defmt_panic as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, OutputType, Speed},
    time::{khz, Hertz},
    timer::{
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
    },
};
use embassy_time::{Duration, Timer};
use panic_probe as _;

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
    let ch1_pin = PwmPin::new(p.PA6, OutputType::PushPull);
    let ch2_pin = PwmPin::new(p.PA7, OutputType::PushPull);
    let ch3_pin = PwmPin::new(p.PB0, OutputType::PushPull);
    let mut pwm_driver = SimplePwm::new(
        p.TIM3,
        Some(ch1_pin),
        Some(ch2_pin),
        Some(ch3_pin),
        None,
        khz(16),
        CountingMode::CenterAlignedUpInterrupts,
    );
    pwm_driver.ch1().enable();
    pwm_driver.ch2().enable();
    pwm_driver.ch3().enable();

    let mut led = Output::new(p.PC14, Level::Low, Speed::Low);
    enable_pin.set_high();

    let delay: u64 = 100;
    let duty: u16 = pwm_driver.max_duty_cycle() / 32;

    loop {
        led.toggle();
        pwm_driver.ch1().set_duty_cycle(duty);
        pwm_driver.ch2().set_duty_cycle_fully_off();
        pwm_driver.ch3().set_duty_cycle_fully_off();
        Timer::after(Duration::from_millis(delay)).await;

        led.toggle();
        pwm_driver.ch1().set_duty_cycle(duty);
        pwm_driver.ch2().set_duty_cycle(duty);
        pwm_driver.ch3().set_duty_cycle_fully_off();
        Timer::after(Duration::from_millis(delay)).await;

        //led.toggle();
        //pwm_driver.ch1().set_duty_cycle_fully_off();
        //pwm_driver.ch2().set_duty_cycle(duty);
        //pwm_driver.ch3().set_duty_cycle_fully_off();
        //Timer::after(Duration::from_millis(delay)).await;

        //led.toggle();
        //pwm_driver.ch1().set_duty_cycle_fully_off();
        //pwm_driver.ch2().set_duty_cycle(duty);
        //pwm_driver.ch3().set_duty_cycle(duty);
        //Timer::after(Duration::from_millis(delay)).await;

        //led.toggle();
        //pwm_driver.ch1().set_duty_cycle_fully_off();
        //pwm_driver.ch2().set_duty_cycle_fully_off();
        //pwm_driver.ch3().set_duty_cycle(duty);
        //Timer::after(Duration::from_millis(delay)).await;

        //led.toggle();
        //pwm_driver.ch1().set_duty_cycle(duty);
        //pwm_driver.ch2().set_duty_cycle_fully_off();
        //pwm_driver.ch3().set_duty_cycle(duty);
        //Timer::after(Duration::from_millis(delay)).await;
    }
}
