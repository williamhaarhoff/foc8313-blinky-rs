#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::pac::timer::{regs::Ccr1ch, vals::Mms};
use embassy_stm32::time::{khz, Hertz};
use embassy_stm32::timer::low_level::{OutputCompareMode, Timer as LLTimer};
use embassy_stm32::timer::{Ch1, Ch2, Ch3, Ch4, Channel, GeneralInstance4Channel, TimerPin};
use embassy_stm32::{
    gpio::{AfType, Flex, Level, Output, OutputType, Speed},
    interrupt, pac,
};
use embassy_stm32::{Config, Peri};
use embassy_time::{Duration, Timer};
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
    pac::TIM3.sr().modify(|r| r.set_uif(false));
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

    let adc = pac::ADC1;
    let rcc = pac::RCC;
    rcc.apb2enr().modify(|w| w.set_adc1en(true));

    adc.cr2().modify(|w| w.set_adon(true));

    Timer::after(Duration::from_millis(100)).await;

    // reset calibration
    adc.cr2().modify(|w| w.set_rstcal(true));
    while adc.cr2().read().rstcal() {
        // spin
    }

    // calibrate
    adc.cr2().modify(|w| w.set_cal(true));
    while adc.cr2().read().cal() {
        // spin
    }
    Timer::after(Duration::from_millis(100)).await;

    adc.cr1().modify(|w| w.set_scan(true));
    adc.cr2().modify(|w| w.set_cont(false));
    adc.cr1().modify(|w| w.set_discen(false));
    adc.cr2().modify(|w| w.set_extsel(0b111)); // ADC SOFTWARE START
    adc.cr2().modify(|w| w.set_align(true));
    // set number of conversions to ?

    adc.cr1().modify(|w| w.set_jdiscen(false));
    adc.cr2().modify(|w| w.set_jextsel(0b100)); // TIM3 CC4 event
    adc.cr1().modify(|w| w.set_jauto(false));

    Timer::after(Duration::from_millis(1000)).await;
    defmt::info!("adc.cr2: {:?}", adc.cr2().read());
    defmt::info!("adc.cr1: {:?}", adc.cr1().read());

    // configure injected channels
    adc.jsqr().modify(|w| w.set_jl(1)); // 2 conversions
    adc.jsqr().modify(|w| w.set_jsq(0, 4)); // JSQ3[4:0] = ADC_CHANNEL_4
    adc.jsqr().modify(|w| w.set_jsq(1, 5)); // JSQ4[4:0] = ADC_CHANNEL_5
    Timer::after(Duration::from_millis(100)).await;

    let raw = unsafe { core::ptr::read_volatile(&adc.jsqr() as *const _ as *const u32) };

    defmt::info!("adc.jsqr: {=u32:#010x}", raw);

    let mut enable_pin = Output::new(p.PB1, Level::Low, Speed::Low);
    //let mut led = Output::new(p.PC14, Level::Low, Speed::Low);
    enable_pin.set_high();

    let mut pwm_driver = MyPwm::new(p.TIM3, p.PA6, p.PA7, p.PB0, khz(16));
    pwm_driver.enable(Channel::Ch1);
    pwm_driver.enable(Channel::Ch2);
    pwm_driver.enable(Channel::Ch3);
    let duty = pwm_driver.get_max_duty() / 4;

    unsafe {
        //cortex_m::peripheral::NVIC::unmask(interrupt::TIM3);
    }

    loop {
        //led.set_high();
        pwm_driver.set_duty(Channel::Ch1, duty);
        Timer::after(Duration::from_millis(500)).await;

        //led.set_low();
        pwm_driver.set_duty(Channel::Ch1, 0);
        Timer::after(Duration::from_millis(1000)).await;
    }
}

pub struct MyPwm<'d, T: GeneralInstance4Channel> {
    tim: LLTimer<'d, T>,
    _ph1: Flex<'d>,
    _ph2: Flex<'d>,
    _ph3: Flex<'d>,
}

impl<'d, T: GeneralInstance4Channel> MyPwm<'d, T> {
    pub fn new(
        tim: Peri<'d, T>,
        ch1: Peri<'d, impl TimerPin<T, Ch1>>,
        ch2: Peri<'d, impl TimerPin<T, Ch2>>,
        ch3: Peri<'d, impl TimerPin<T, Ch3>>,
        freq: Hertz,
    ) -> Self {
        let af1 = ch1.af_num();
        let af2 = ch2.af_num();
        let af3 = ch3.af_num();
        let mut ch1 = Flex::new(ch1);
        let mut ch2 = Flex::new(ch2);
        let mut ch3 = Flex::new(ch3);
        ch1.set_as_af_unchecked(af1, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        ch2.set_as_af_unchecked(af2, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        ch3.set_as_af_unchecked(af3, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        let mut this = Self {
            tim: LLTimer::new(tim),
            _ph1: ch1,
            _ph2: ch2,
            _ph3: ch3,
        };

        this.set_frequency(freq);
        this.tim.start();

        [Channel::Ch1, Channel::Ch2, Channel::Ch3]
            .iter()
            .for_each(|&channel| {
                this.tim
                    .set_output_compare_mode(channel, OutputCompareMode::PwmMode1);
                this.tim.set_output_compare_preload(channel, true);
            });

        // configure Ch4 to generate interrupts on cc event
        this.tim
            .set_output_compare_mode(Channel::Ch4, OutputCompareMode::Frozen);
        this.tim.set_output_compare_preload(Channel::Ch4, true);
        this.tim.regs_gp16().dier().modify(|w| {
            w.set_ccie(3, true);
        });

        // configure master mode, event generation
        this.tim.regs_gp16().cr2().modify(|w| {
            w.set_mms(Mms::COMPARE_OC4);
        });

        this
    }

    pub fn enable(&mut self, channel: Channel) {
        self.tim
            .regs_gp16()
            .ccer()
            .modify(|w| w.set_cce(channel.index(), true));
    }

    pub fn disable(&mut self, channel: Channel) {
        self.tim
            .regs_gp16()
            .ccer()
            .modify(|w| w.set_cce(channel.index(), false));
    }

    pub fn set_frequency(&mut self, freq: Hertz) {
        let multiplier = if self.tim.get_counting_mode().is_center_aligned() {
            2u8
        } else {
            1u8
        };
        self.tim.set_frequency(freq * multiplier);
    }

    pub fn get_max_duty(&self) -> u16 {
        let max = self.tim.get_max_compare_value();
        max as u16
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        defmt::assert!(duty < self.get_max_duty());

        self.tim
            .regs_gp16()
            .ccr(channel.index())
            .write_value(Ccr1ch(duty as u32));
    }
}
