use core::marker::PhantomData;
use embassy_stm32::gpio::{AfType, Flex, OutputType, Speed};
use embassy_stm32::pac::timer::regs::Ccr1ch;
pub use embassy_stm32::pac::timer::vals::Mms;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, OutputCompareMode, Timer as LLTimer};
use embassy_stm32::timer::{Channel, GeneralInstance4Channel, TimerChannel, TimerPin};
use embassy_stm32::Peri;

#[derive(Clone, Copy)]
pub enum Phase {
    A,
    B,
    C,
}

pub enum MaybeChannel {
    Valid(Channel),
    Invalid,
}

// Output events
pub trait TriggerOut {
    const MODE: Mms;
    const CHANNEL: MaybeChannel;
}

pub struct Reset;
pub struct Enable;
pub struct Update;
pub struct ComparePulse;
pub struct CompareOC1;
pub struct CompareOC2;
pub struct CompareOC3;
pub struct CompareOC4;
impl TriggerOut for Reset {
    const MODE: Mms = Mms::RESET;
    const CHANNEL: MaybeChannel = MaybeChannel::Invalid;
}
impl TriggerOut for Enable {
    const MODE: Mms = Mms::ENABLE;
    const CHANNEL: MaybeChannel = MaybeChannel::Invalid;
}
impl TriggerOut for Update {
    const MODE: Mms = Mms::UPDATE;
    const CHANNEL: MaybeChannel = MaybeChannel::Invalid;
}
impl TriggerOut for ComparePulse {
    const MODE: Mms = Mms::COMPARE_PULSE;
    const CHANNEL: MaybeChannel = MaybeChannel::Invalid;
}
impl TriggerOut for CompareOC1 {
    const MODE: Mms = Mms::COMPARE_OC1;
    const CHANNEL: MaybeChannel = MaybeChannel::Valid(Channel::Ch1);
}
impl TriggerOut for CompareOC2 {
    const MODE: Mms = Mms::COMPARE_OC2;
    const CHANNEL: MaybeChannel = MaybeChannel::Valid(Channel::Ch2);
}
impl TriggerOut for CompareOC3 {
    const MODE: Mms = Mms::COMPARE_OC3;
    const CHANNEL: MaybeChannel = MaybeChannel::Valid(Channel::Ch3);
}
impl TriggerOut for CompareOC4 {
    const MODE: Mms = Mms::COMPARE_OC4;
    const CHANNEL: MaybeChannel = MaybeChannel::Valid(Channel::Ch4);
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
        E: TriggerOut,
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

        // configure trigger out that is also a timer channel to generate interrupts on cc event
        // todo - prevent trigger out on timer channel conflicting with channel used by phase
        match E::CHANNEL {
            MaybeChannel::Valid(chx) => {
                this.tim
                    .set_output_compare_mode(chx, OutputCompareMode::Toggle);
                this.tim.set_output_compare_preload(chx, true);
            }
            MaybeChannel::Invalid => {}
        }

        // todo - allow timer interrupts?
        //this.tim.regs_gp16().dier().modify(|w| {
        //    w.set_ccie(3, true);
        //});

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
