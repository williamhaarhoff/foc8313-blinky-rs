use core::marker::PhantomData;
use embassy_stm32::interrupt;
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::{adc::SampleTime, Peri};
use embassy_sync::waitqueue::AtomicWaker;

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
    #[allow(unused)]
    sample_time: SampleTime,
}

pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::state().waker.wake();
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
