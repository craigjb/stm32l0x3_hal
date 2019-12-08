use crate::gpio::gpioa::{PA13, PA14, PA2, PA3};
use crate::gpio::gpiob::{PB10, PB11};
use crate::gpio::gpioc::{PC0, PC1, PC10, PC11, PC4, PC5};
use crate::gpio::{AF0, AF2, AF4, AF6, AF7};
use crate::rcc::{Clocks, LpUsartClock, APB1, CCIPR};
use stm32l0x3::LPUSART1;

pub trait LpUsartExt {
    fn constrain<TX, RX>(self, tx_pin: TX, rx_pin: RX) -> LpUsart<TX, RX>
    where
        TX: LpUsartTxPin,
        RX: LpUsartRxPin;
}

impl LpUsartExt for LPUSART1 {
    fn constrain<TX, RX>(self, tx_pin: TX, rx_pin: RX) -> LpUsart<TX, RX>
    where
        TX: LpUsartTxPin,
        RX: LpUsartRxPin,
    {
        LpUsart::<TX, RX> {
            tx_pin,
            rx_pin
        }
    }
}

pub unsafe trait LpUsartTxPin {}
pub unsafe trait LpUsartRxPin {}

unsafe impl LpUsartTxPin for PA2<AF6> {}
unsafe impl LpUsartRxPin for PA3<AF6> {}

unsafe impl LpUsartTxPin for PA14<AF6> {}
unsafe impl LpUsartRxPin for PA13<AF6> {}

unsafe impl LpUsartTxPin for PB10<AF4> {}
unsafe impl LpUsartRxPin for PB11<AF4> {}
unsafe impl LpUsartTxPin for PB11<AF7> {}
unsafe impl LpUsartRxPin for PB10<AF7> {}

unsafe impl LpUsartTxPin for PC1<AF6> {}
unsafe impl LpUsartRxPin for PC0<AF6> {}

unsafe impl LpUsartTxPin for PC4<AF2> {}
unsafe impl LpUsartRxPin for PC5<AF2> {}

unsafe impl LpUsartTxPin for PC10<AF0> {}
unsafe impl LpUsartRxPin for PC11<AF0> {}

pub struct LpUsart<TX, RX>
where
    TX: LpUsartTxPin,
    RX: LpUsartRxPin
 {
    tx_pin: TX,
    rx_pin: RX
}

impl<TX, RX> LpUsart<TX, RX>
where
    TX: LpUsartTxPin,
    RX: LpUsartRxPin
{
    pub fn configure(
        &mut self,
        config: &LpUsartConfig,
        clocks: &Clocks,
        apb1: &mut APB1,
        ccipr: &mut CCIPR,
    ) {
        ccipr.set_lpusart_clock(LpUsartClock::SystemClock);
        apb1.enr().modify(|_, w| w.lpuart1en().set_bit());
        apb1.rstr().modify(|_, w| w.lpuart1rst().set_bit());
        apb1.rstr().modify(|_, w| w.lpuart1rst().clear_bit());

        let div: u32 = (clocks.sysclk().0 << 6) / config.baud_rate;
        let div = (div * 256) >> 6;

        let regs = unsafe { &(*LPUSART1::ptr()) };
        let (m1, m0) = config.word_length.lpuart_cr1_bits();
        regs.cr1
            .modify(|_, w| w.m1().bit(m1).m0().bit(m0).ps().bit(config.parity));
        regs.brr.write(|w| unsafe { w.bits(div) });
        regs.cr2
            .modify(|_, w| unsafe { w.stop().bits(config.stop_bits.lpuart_cr2_bits()) });
        regs.cr3.modify(|_, w| w.ovrdis().set_bit());
        regs.cr1.modify(|_, w| w.ue().set_bit().re().set_bit().te().set_bit());
    }

    pub fn enable_rx_interrupt(&mut self) {
        unsafe { &(*LPUSART1::ptr()).cr1.modify(|_, w| w.rxneie().set_bit()) };
    }

    pub fn disable_rx_interrupt(&mut self) {
        unsafe { &(*LPUSART1::ptr()).cr1.modify(|_, w| w.rxneie().clear_bit()) };
    }

    pub fn enable_tx_interrupt(&mut self) {
        unsafe { &(*LPUSART1::ptr()).cr1.modify(|_, w| w.txeie().set_bit()) };
    }

    pub fn disable_tx_interrupt(&mut self) {
        unsafe { &(*LPUSART1::ptr()).cr1.modify(|_, w| w.txeie().clear_bit()) };
    }

    pub fn is_transmitting(&self) -> bool {
        unsafe { (*LPUSART1::ptr()).isr.read().txe().bit_is_clear() }
    }

    pub fn get_received_byte(&mut self) -> Option<u8> {
        let regs = unsafe { &(*LPUSART1::ptr()) };
        if regs.isr.read().rxne().bit_is_set() {
            Some(regs.rdr.read().rdr().bits() as u8)
        } else {
            None
        }
    }

    pub fn transmit_byte(&mut self, b: u8) {
        let regs = unsafe { &(*LPUSART1::ptr()) };
        regs.tdr.write(|w| unsafe { w.tdr().bits(b as u16) });
    }
}

pub enum WordLength {
    Word8Bits,
    Word9Bits,
    Word7Bits,
}

impl WordLength {
    fn lpuart_cr1_bits(&self) -> (bool, bool) {
        match self {
            Word8Bits => (false, false),
            Word9Bits => (false, true),
            Word7Bits => (true, false),
        }
    }
}

pub enum StopBits {
    StopBits1,
    StopBits2,
}

impl StopBits {
    fn lpuart_cr2_bits(&self) -> u8 {
        match self {
            StopBits1 => 0b00,
            StopBits2 => 0b10,
        }
    }
}

pub struct LpUsartConfig {
    word_length: WordLength,
    parity: bool,
    stop_bits: StopBits,
    baud_rate: u32,
}

impl LpUsartConfig {
    pub fn new() -> Self {
        LpUsartConfig {
            word_length: WordLength::Word8Bits,
            parity: false,
            stop_bits: StopBits::StopBits1,
            baud_rate: 115200,
        }
    }

    pub fn word_length(mut self, word_length: WordLength) -> Self {
        self.word_length = word_length;
        self
    }

    pub fn parity(mut self, parity: bool) -> Self {
        self.parity = parity;
        self
    }

    pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    pub fn baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }
}
