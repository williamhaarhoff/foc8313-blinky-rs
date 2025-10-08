use core::marker::PhantomData;
#[allow(unused_imports)]
use defmt::*;
use embassy_stm32::gpio::{AfType, Flex, Level, Output, OutputType, Speed};
use embassy_stm32::pac::timer::regs::Ccr1ch;
pub use embassy_stm32::pac::timer::vals::Mms;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, OutputCompareMode, Timer as LLTimer};
use embassy_stm32::timer::{
    Ch1, Ch2, Ch3, Ch4, Channel, GeneralInstance4Channel, TimerChannel, TimerPin,
};
use embassy_stm32::Peri;

#[derive(Clone, Copy)]
pub enum Phase {
    A,
    B,
    C,
}

pub struct SomeChannel<C>(core::marker::PhantomData<C>);
pub struct NoChannel;

pub trait MaybeChannel {}
impl<C> MaybeChannel for SomeChannel<C> {}
impl MaybeChannel for NoChannel {}

// Output events
pub trait TriggerOut {
    const MODE: Mms;
    type Channel;
}

struct Reset;
struct Enable;
struct Update;
struct ComparePulse;
struct CompareOC1;
struct CompareOC2;
struct CompareOC3;
struct CompareOC4;
impl TriggerOut for Reset {
    const MODE: Mms = Mms::RESET;
    type Channel = NoChannel;
}
impl TriggerOut for Enable {
    const MODE: Mms = Mms::ENABLE;
    type Channel = NoChannel;
}
impl TriggerOut for Update {
    const MODE: Mms = Mms::UPDATE;
    type Channel = NoChannel;
}
impl TriggerOut for ComparePulse {
    const MODE: Mms = Mms::COMPARE_PULSE;
    type Channel = NoChannel;
}
impl TriggerOut for CompareOC1 {
    const MODE: Mms = Mms::COMPARE_OC1;
    type Channel = SomeChannel<Ch1>;
}
impl TriggerOut for CompareOC2 {
    const MODE: Mms = Mms::COMPARE_OC2;
    type Channel = SomeChannel<Ch2>;
}
impl TriggerOut for CompareOC3 {
    const MODE: Mms = Mms::COMPARE_OC3;
    type Channel = SomeChannel<Ch3>;
}
impl TriggerOut for CompareOC4 {
    const MODE: Mms = Mms::COMPARE_OC4;
    type Channel = SomeChannel<Ch4>;
}

pub trait NotSame<T> {}
impl<T, U> NotSame<U> for T {}
impl<T> NotSame<T> for T {}

pub trait ChannelNotConflictingWith<A, B, C> {}
impl<A, B, C> ChannelNotConflictingWith<A, B, C> for NoChannel {}
impl<Cx, A, B, C> ChannelNotConflictingWith<A, B, C> for SomeChannel<Cx> where
    Cx: NotSame<A> + NotSame<B> + NotSame<C>
{
}

pub struct Pwm3<'d, T: GeneralInstance4Channel, A: TimerChannel, B: TimerChannel, C: TimerChannel> {
    tim: LLTimer<'d, T>,
    _cha: Flex<'d>,
    _chb: Flex<'d>,
    _chc: Flex<'d>,
    _a: PhantomData<A>,
    _b: PhantomData<B>,
    _c: PhantomData<C>,
}

impl<'d, T: GeneralInstance4Channel, A: TimerChannel, B: TimerChannel, C: TimerChannel>
    Pwm3<'d, T, A, B, C>
{
    pub fn new<E>(
        tim: Peri<'d, T>,
        cha: Peri<'d, impl TimerPin<T, A>>,
        chb: Peri<'d, impl TimerPin<T, B>>,
        chc: Peri<'d, impl TimerPin<T, C>>,
        _trg: E,
        freq: Hertz,
    ) -> Self
    where
        E: TriggerOut + ChannelNotConflictingWith<A, B, C>,
    {
        let afa = cha.af_num();
        let afb = chb.af_num();
        let afc = chc.af_num();
        let mut cha = Flex::new(cha);
        let mut chb = Flex::new(chb);
        let mut chc = Flex::new(chc);
        cha.set_as_af_unchecked(afa, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        chb.set_as_af_unchecked(afb, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        chc.set_as_af_unchecked(afc, AfType::output(OutputType::PushPull, Speed::VeryHigh));

        let mut this = Self {
            tim: LLTimer::new(tim),
            _cha: cha,
            _chb: chb,
            _chc: chc,
            _a: PhantomData,
            _b: PhantomData,
            _c: PhantomData,
        };
        this.tim
            .set_counting_mode(CountingMode::CenterAlignedUpInterrupts);

        this.set_frequency(freq);
        this.tim.start();

        [A::CHANNEL, B::CHANNEL, C::CHANNEL]
            .iter()
            .for_each(|&channel| {
                this.tim
                    .set_output_compare_mode(channel, OutputCompareMode::PwmMode1);
                this.tim.set_output_compare_preload(channel, true);
            });

        // configure Ch4 to generate interrupts on cc event

        this.tim
            .set_output_compare_mode(Channel::Ch4, OutputCompareMode::Toggle);
        this.tim.set_output_compare_preload(Channel::Ch4, true);
        this.tim.regs_gp16().dier().modify(|w| {
            w.set_ccie(3, true);
        });

        // configure master mode, event generation
        this.tim.regs_gp16().cr2().modify(|w| {
            w.set_mms(E::MODE);
        });
        this
    }

    pub fn enable(&mut self, phase: Phase) {
        let channel = match phase {
            Phase::A => A::CHANNEL,
            Phase::B => B::CHANNEL,
            Phase::C => C::CHANNEL,
        };
        self.tim
            .regs_gp16()
            .ccer()
            .modify(|w| w.set_cce(channel.index(), true));
    }

    pub fn disable(&mut self, phase: Phase) {
        let channel = match phase {
            Phase::A => A::CHANNEL,
            Phase::B => B::CHANNEL,
            Phase::C => C::CHANNEL,
        };
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

    pub fn set_duty(&mut self, phase: Phase, duty: u16) {
        let channel = match phase {
            Phase::A => A::CHANNEL,
            Phase::B => B::CHANNEL,
            Phase::C => C::CHANNEL,
        };
        defmt::assert!(duty < self.get_max_duty());
        self.tim
            .regs_gp16()
            .ccr(channel.index())
            .write_value(Ccr1ch(duty as u32));
    }
}
