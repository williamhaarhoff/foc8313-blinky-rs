#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::Adc;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::{adc, bind_interrupts};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1);
    let mut sense_c = p.PA3;
    let mut sense_b = p.PA4;

    let mut drv_enable = Output::new(p.PB1, Level::Low, Speed::Low);
    let mut drv_a = Output::new(p.PA6, Level::Low, Speed::Low);
    let mut drv_b = Output::new(p.PA7, Level::Low, Speed::Low);
    let mut drv_c = Output::new(p.PB0, Level::Low, Speed::Low);

    let mut vrefint = adc.enable_vref();
    let vrefint_sample = adc.read(&mut vrefint).await;
    let convert_to_millivolts = |sample| {
        // From http://www.st.com/resource/en/datasheet/CD00161566.pdf
        // 5.3.4 Embedded reference voltage
        const VREFINT_MV: u32 = 1200; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    drv_enable.set_high();
    let mut vc = adc.read(&mut sense_c).await;
    let mut vb = adc.read(&mut sense_b).await;

    info!("pre loop values!");
    info!(
        " a: {} b: {} {} {}mV c: {} {} {}mV",
        drv_a.is_set_high(),
        drv_b.is_set_high(),
        vb,
        convert_to_millivolts(vb),
        drv_c.is_set_high(),
        vc,
        convert_to_millivolts(vc)
    );

    drv_a.set_high();
    drv_b.set_low();
    drv_c.set_low();
    Timer::after_millis(1000).await;
    vc = adc.read(&mut sense_c).await;
    vb = adc.read(&mut sense_b).await;
    info!(
        " a: {} b: {} {} {}mV c: {} {} {}mV",
        drv_a.is_set_high(),
        drv_b.is_set_high(),
        vb,
        convert_to_millivolts(vb),
        drv_c.is_set_high(),
        vc,
        convert_to_millivolts(vc)
    );

    drv_a.set_high();
    drv_b.set_high();
    drv_c.set_low();
    Timer::after_millis(1000).await;
    vc = adc.read(&mut sense_c).await;
    vb = adc.read(&mut sense_b).await;
    info!(
        " a: {} b: {} {} {}mV c: {} {} {}mV",
        drv_a.is_set_high(),
        drv_b.is_set_high(),
        vb,
        convert_to_millivolts(vb),
        drv_c.is_set_high(),
        vc,
        convert_to_millivolts(vc)
    );

    drv_a.set_low();
    drv_b.set_high();
    drv_c.set_low();
    Timer::after_millis(1000).await;
    vc = adc.read(&mut sense_c).await;
    vb = adc.read(&mut sense_b).await;
    info!(
        " a: {} b: {} {} {}mV c: {} {} {}mV",
        drv_a.is_set_high(),
        drv_b.is_set_high(),
        vb,
        convert_to_millivolts(vb),
        drv_c.is_set_high(),
        vc,
        convert_to_millivolts(vc)
    );

    drv_a.set_low();
    drv_b.set_high();
    drv_c.set_high();
    Timer::after_millis(1000).await;
    vc = adc.read(&mut sense_c).await;
    vb = adc.read(&mut sense_b).await;
    info!(
        " a: {} b: {} {} {}mV c: {} {} {}mV",
        drv_a.is_set_high(),
        drv_b.is_set_high(),
        vb,
        convert_to_millivolts(vb),
        drv_c.is_set_high(),
        vc,
        convert_to_millivolts(vc)
    );

    drv_a.set_low();
    drv_b.set_low();
    drv_c.set_high();
    Timer::after_millis(1000).await;
    vc = adc.read(&mut sense_c).await;
    vb = adc.read(&mut sense_b).await;
    info!(
        " a: {} b: {} {} {}mV c: {} {} {}mV",
        drv_a.is_set_high(),
        drv_b.is_set_high(),
        vb,
        convert_to_millivolts(vb),
        drv_c.is_set_high(),
        vc,
        convert_to_millivolts(vc)
    );

    drv_a.set_high();
    drv_b.set_low();
    drv_c.set_high();
    Timer::after_millis(1000).await;
    vc = adc.read(&mut sense_c).await;
    vb = adc.read(&mut sense_b).await;
    info!(
        " a: {} b: {} {} {}mV c: {} {} {}mV",
        drv_a.is_set_high(),
        drv_b.is_set_high(),
        vb,
        convert_to_millivolts(vb),
        drv_c.is_set_high(),
        vc,
        convert_to_millivolts(vc)
    );

    drv_enable.set_low();

    loop {
        Timer::after_millis(1000).await;
    }
}
