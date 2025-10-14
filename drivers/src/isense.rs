use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;
use embassy_stm32::adc::{AdcChannel, AnyAdcChannel};
use embassy_stm32::gpio::Pin;
use embassy_stm32::gpio::{AfType, Flex, OutputType, Speed};
use embassy_stm32::interrupt::typelevel::Interrupt;
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::time::Hertz;
use embassy_stm32::{adc::SampleTime, pac, rcc, Peri};
use embassy_stm32::{interrupt, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

#[allow(unused)]
pub(crate) fn blocking_delay_us(us: u32) {
    {
        embassy_time::block_for(embassy_time::Duration::from_micros(us as u64));
    }
}

pub struct State {
    pub waker: AtomicWaker,
}

impl State {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

pub trait Instance: embassy_stm32::PeripheralType + embassy_stm32::rcc::RccPeripheral {
    #[allow(unused)]
    fn regs() -> embassy_stm32::pac::adc::Adc;
    #[allow(unused)]
    fn state() -> &'static State;
    type Interrupt: embassy_stm32::interrupt::typelevel::Interrupt;
}

pub struct Isense<'d, T: Instance> {
    #[allow(unused)]
    adc: Peri<'d, T>,
    cha: AnyAdcChannel<T>,
    #[allow(unused)]
    sample_time: SampleTime,
}

pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        defmt::info!("adc interrupt!");
        if T::regs().sr().read().jeoc() {
            T::regs().cr1().modify(|w| w.set_jeocie(false));
            T::state().waker.wake();
        }
    }
}

impl<'d, T: Instance> Isense<'d, T> {
    pub fn new(adc: Peri<'d, T>, cha: impl AdcChannel<T>) -> Self {
        rcc::enable_and_reset::<T>();
        T::regs().cr2().modify(|reg| reg.set_adon(true));

        // 11.4: Before starting a calibration, the ADC must have been in power-on state (ADON bit = ‘1’)
        // for at least two ADC clock cycles.
        blocking_delay_us((1_000_000 * 2) / Self::freq().0 + 1);

        // Reset calibration
        T::regs().cr2().modify(|reg| reg.set_rstcal(true));
        while T::regs().cr2().read().rstcal() {
            // spin
        }

        // Calibrate
        T::regs().cr2().modify(|reg| reg.set_cal(true));
        while T::regs().cr2().read().cal() {
            // spin
        }

        // One cycle after calibration
        blocking_delay_us(1_000_000 / Self::freq().0 + 1);

        let cha = cha.degrade_adc();

        // set up scanning injected mode
        T::regs().cr1().modify(|w| w.set_scan(true));
        T::regs().cr2().modify(|w| w.set_cont(false));
        T::regs().cr1().modify(|w| w.set_discen(false));
        T::regs().cr2().modify(|w| w.set_extsel(0b111)); // ADC SOFTWARE START
        T::regs().cr2().modify(|w| w.set_align(false));
        T::regs().cr2().modify(|w| w.set_exttrig(false));
        T::regs().cr2().modify(|w| w.set_jexttrig(true));

        T::regs().cr1().modify(|w| w.set_jdiscen(false));
        //T::regs().cr2().modify(|w| w.set_jextsel(0b100)); // TIM3 CC4 event
        T::regs().cr2().modify(|w| w.set_jextsel(0b111)); // JSWSTART
        T::regs().cr1().modify(|w| w.set_jauto(false));

        // configure injected channels
        T::regs().jsqr().modify(|w| w.set_jl(1)); // 2 conversions
        T::regs().jsqr().modify(|w| w.set_jsq(0, 0)); // JSQ3[4:0] = ADC_CHANNEL_4
        T::regs().jsqr().modify(|w| w.set_jsq(1, 0)); // JSQ4[4:0] = ADC_CHANNEL_5
        T::regs().jsqr().modify(|w| w.set_jsq(2, 3)); // JSQ4[4:0] = ADC_CHANNEL_5
        T::regs().jsqr().modify(|w| w.set_jsq(3, 4)); // JSQ4[4:0] = ADC_CHANNEL_5

        T::regs()
            .smpr2()
            .modify(|w| w.set_smp(5, SampleTime::CYCLES1_5));
        T::regs()
            .smpr2()
            .modify(|w| w.set_smp(4, SampleTime::CYCLES1_5));

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            adc,
            sample_time: SampleTime::from_bits(0),
            cha,
        }
    }

    fn freq() -> Hertz {
        rcc::frequency::<T>()
    }

    pub fn sample_time_for_us(&self, us: u32) -> SampleTime {
        match us * Self::freq().0 / 1_000_000 {
            0..=1 => SampleTime::CYCLES1_5,
            2..=7 => SampleTime::CYCLES7_5,
            8..=13 => SampleTime::CYCLES13_5,
            14..=28 => SampleTime::CYCLES28_5,
            29..=41 => SampleTime::CYCLES41_5,
            42..=55 => SampleTime::CYCLES55_5,
            56..=71 => SampleTime::CYCLES71_5,
            _ => SampleTime::CYCLES239_5,
        }
    }

    //pub fn enable_vref(&self) -> Vref {
    //    T::regs().cr2().modify(|reg| {
    //        reg.set_tsvrefe(true);
    //    });
    //    Vref {}
    //}

    //pub fn enable_temperature(&self) -> Temperature {
    //    T::regs().cr2().modify(|reg| {
    //        reg.set_tsvrefe(true);
    //    });
    //    Temperature {}
    //}

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    /// Perform a single conversion.
    pub async fn convert(&mut self) -> u16 {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
            reg.set_jswstart(true);
        });
        T::regs().cr1().modify(|w| w.set_eocie(true));
        T::regs().cr1().modify(|w| w.set_jeocie(true));

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if T::regs().sr().read().jeoc() {
                defmt::info!("polling!");
                Poll::Ready(())
            } else {
                defmt::info!("pending");
                Poll::Pending
            }
        })
        .await;

        T::regs().sr().modify(|w| w.set_jeoc(false));
        T::regs().sr().modify(|w| w.set_eoc(false));
        T::regs().sr().modify(|w| w.set_jstrt(false));
        T::regs().sr().modify(|w| w.set_strt(false));

        T::regs().jdr(0).read().0 as u16
    }

    // pub async fn read(&mut self, channel: &mut impl AdcChannel<T>) -> u16 {
    //     Self::set_channel_sample_time(channel.channel(), self.sample_time);
    //     T::regs().cr1().modify(|reg| {
    //         reg.set_scan(false);
    //         reg.set_discen(false);
    //     });
    //     T::regs().sqr1().modify(|reg| reg.set_l(0));

    //     T::regs().cr2().modify(|reg| {
    //         reg.set_cont(false);
    //         reg.set_exttrig(true);
    //         reg.set_swstart(false);
    //         reg.set_extsel(7); // SWSTART
    //     });

    //     // Configure the channel to sample
    //     T::regs()
    //         .sqr3()
    //         .write(|reg| reg.set_sq(0, channel.channel()));
    //     self.convert().await
    // }

    fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        if ch <= 9 {
            T::regs()
                .smpr2()
                .modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs()
                .smpr1()
                .modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }
}

impl<'d, T: Instance> Drop for Isense<'d, T> {
    fn drop(&mut self) {
        T::regs().cr2().modify(|reg| reg.set_adon(false));

        rcc::disable::<T>();
    }
}

// manually created instance for f1
impl Instance for ADC1 {
    fn regs() -> embassy_stm32::pac::adc::Adc {
        embassy_stm32::pac::ADC1
    }
    fn state() -> &'static State {
        static STATE: State = State::new();
        &STATE
    }
    type Interrupt = embassy_stm32::interrupt::typelevel::ADC1_2;
}
