//! Reset and Clock Control

use crate::flash::ACR;
use crate::time::Hertz;
use stm32l0x3::{rcc, RCC};

/// Extension trait that constrains the `RCC` peripheral
pub trait RccExt {
    /// Constrains the `RCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Rcc;
}

impl RccExt for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            ahb: AHB { _0: () },
            apb1: APB1 { _0: () },
            apb2: APB2 { _0: () },
            gpio: GPIO { _0: () },
            cfgr: CFGR::new(),
            ccipr: CCIPR::new(),
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    /// AMBA High-performance Bus (AHB) registers
    pub ahb: AHB,
    /// Advanced Peripheral Bus 1 (APB1) registers
    pub apb1: APB1,
    /// Advanced Peripheral Bus 2 (APB2) registers
    pub apb2: APB2,
    /// GPIO registers
    pub gpio: GPIO,
    /// Clock configuration
    pub cfgr: CFGR,
    /// Clock configuration
    pub ccipr: CCIPR,
}

/// AMBA High-performance Bus (AHB) registers
pub struct AHB {
    _0: (),
}

impl AHB {
    pub(crate) fn enr(&mut self) -> &rcc::AHBENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).ahbenr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::AHBRSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).ahbrstr }
    }
}

/// Advanced Peripheral Bus 1 (APB1) registers
pub struct APB1 {
    _0: (),
}

impl APB1 {
    pub(crate) fn enr(&mut self) -> &rcc::APB1ENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb1enr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::APB1RSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb1rstr }
    }
}

/// Advanced Peripheral Bus 2 (APB2) registers
pub struct APB2 {
    _0: (),
}

impl APB2 {
    pub(crate) fn enr(&mut self) -> &rcc::APB2ENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb2enr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::APB2RSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).apb2rstr }
    }
}

/// GPIO RCC registers
pub struct GPIO {
    _0: (),
}

impl GPIO {
    pub(crate) fn enr(&mut self) -> &rcc::IOPENR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).iopenr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::IOPRSTR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).ioprstr }
    }
}

pub enum LpUsartClock {
    ApbClock,
    SystemClock,
    HSI16Clock,
    LSEClock,
}

impl LpUsartClock {
    fn ccipr_bits(&self) -> (bool, bool) {
        match self {
            LpUsartClock::ApbClock => (false, false),
            LpUsartClock::SystemClock => (false, true),
            LpUsartClock::HSI16Clock => (true, false),
            LpUsartClock::LSEClock => (true, true),
        }
    }
}

pub struct CCIPR {}

impl CCIPR {
    pub fn new() -> Self {
        Self {}
    }

    pub fn set_lpusart_clock(&mut self, source: LpUsartClock) {
        let (sel1, sel0) = source.ccipr_bits();
        unsafe {
            &(*RCC::ptr())
                .ccipr
                .modify(|_, w| w.lpuart1sel1().bit(sel1).lpuart1sel0().bit(sel0));
        }
    }
}

const HSI: u32 = 16_000_000; // Hz
const USB_PLL_FREQ: u32 = 96_000_000; // Hz

pub enum ExternalHseType {
    Clock,
    Crystal,
}

/// Clock configuration
pub struct CFGR {
    hse: Option<(ExternalHseType, u32)>,
    usb_pll: bool,
    hclk: Option<u32>,
    pclk1: Option<u32>,
    pclk2: Option<u32>,
    sysclk: Option<u32>,
}

impl CFGR {
    fn new() -> CFGR {
        CFGR {
            hse: None,
            usb_pll: false,
            hclk: None,
            pclk1: None,
            pclk2: None,
            sysclk: None,
        }
    }

    /// Use an external oscillator instead of HSI
    ///
    /// With an external clock, max 32 MHz; with an external crystal max 24 MHz
    pub fn external_hse<F>(mut self, ctype: ExternalHseType, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.hse = Some((ctype, freq.into().0));
        self
    }

    pub fn usb_pll(mut self, enabled: bool) -> Self {
        self.usb_pll = enabled;
        self
    }

    /// Sets a frequency for the AHB bus
    pub fn hclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.hclk = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB1 bus
    pub fn pclk1<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.pclk1 = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB2 bus
    pub fn pclk2<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.pclk2 = Some(freq.into().0);
        self
    }

    /// Sets the system (core) frequency
    pub fn sysclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.sysclk = Some(freq.into().0);
        self
    }

    /// Freezes the clock configuration, making it effective
    pub fn freeze(self, acr: &mut ACR) -> Clocks {
        let (hse_type, hse_freq) = self
            .hse
            .map_or((None, None), |hse| (Some(hse.0), Some(hse.1)));
        let pll_in_freq = hse_freq.unwrap_or(HSI);
        let pll_freq = if self.usb_pll {
            USB_PLL_FREQ
        } else {
            2 * self.sysclk.unwrap_or(hse_freq.unwrap_or(HSI))
        };

        let sysclk_freq = self.sysclk.unwrap_or(if pll_freq > 96_000_000 {
            pll_freq / 4
        } else if pll_freq > 64_000_000 {
            pll_freq / 3
        } else {
            pll_freq / 2
        });

        let pll_mul = pll_freq / pll_in_freq;
        let pll_div = pll_freq / sysclk_freq;

        let pll_mul_div_bits = if pll_mul == 2 && pll_div == 2 && !self.usb_pll {
            None
        } else {
            let mul: u8 = match pll_mul {
                3 => 0b0000,
                4 => 0b0001,
                6 => 0b0010,
                8 => 0b0011,
                12 => 0b0100,
                16 => 0b0101,
                24 => 0b0110,
                32 => 0b0111,
                48 => 0b1000,
                _ => unreachable!(),
            };
            let div: u8 = match pll_div {
                m @ 2..=4 => m as u8 - 1,
                _ => unreachable!(),
            };
            Some((mul, div))
        };

        match hse_type {
            Some(ExternalHseType::Clock) => assert!(sysclk_freq <= 32_000_000),
            Some(ExternalHseType::Crystal) => assert!(sysclk_freq <= 24_000_000),
            _ => {}
        };

        let hpre_bits = self
            .hclk
            .map(|hclk| match sysclk_freq / hclk {
                0 => unreachable!(),
                1 => 0b0111,
                2 => 0b1000,
                3..=5 => 0b1001,
                6..=11 => 0b1010,
                12..=39 => 0b1011,
                40..=95 => 0b1100,
                96..=191 => 0b1101,
                192..=383 => 0b1110,
                _ => 0b1111,
            })
            .unwrap_or(0b0111);

        let hclk = sysclk_freq / (1 << (hpre_bits - 0b0111));
        match hse_type {
            Some(ExternalHseType::Clock) => assert!(hclk <= 32_000_000),
            Some(ExternalHseType::Crystal) => assert!(hclk <= 24_000_000),
            _ => {}
        };

        let ppre1_bits: u8 = self
            .pclk1
            .map(|pclk1| match hclk / pclk1 {
                0 => unreachable!(),
                1 => 0b011,
                2 => 0b100,
                3..=5 => 0b101,
                6..=11 => 0b110,
                _ => 0b111,
            })
            .unwrap_or(0b011);

        let ppre1 = 1 << (ppre1_bits - 0b011);
        let pclk1 = hclk / ppre1 as u32;

        match hse_type {
            Some(ExternalHseType::Clock) => assert!(pclk1 <= 32_000_000),
            Some(ExternalHseType::Crystal) => assert!(pclk1 <= 24_000_000),
            _ => {}
        };

        let ppre2_bits: u8 = self
            .pclk2
            .map(|pclk2| match hclk / pclk2 {
                0 => unreachable!(),
                1 => 0b011,
                2 => 0b100,
                3..=5 => 0b101,
                6..=11 => 0b110,
                _ => 0b111,
            })
            .unwrap_or(0b011);

        let ppre2 = 1 << (ppre2_bits - 0b011);
        let pclk2 = hclk / ppre2 as u32;

        match hse_type {
            Some(ExternalHseType::Clock) => assert!(pclk2 <= 32_000_000),
            Some(ExternalHseType::Crystal) => assert!(pclk2 <= 24_000_000),
            _ => {}
        };

        // Adjust flash wait states
        acr.acr().write(|w| {
            // In Range 1, frequencies 16 MHz and below don't require wait states
            if sysclk_freq <= 16_000_000 {
                w.latency().clear_bit()
            } else {
                w.latency().set_bit()
            }
        });

        let hse_en = match hse_type {
            Some(_) => true,
            None => false,
        };

        let rcc = unsafe { &*RCC::ptr() };
        if let Some((pllmul_bits, plldiv_bits)) = pll_mul_div_bits {
            // use PLL as source
            // turn off PLL and wait until it's not ready
            rcc.cr.write(|w| w.pllon().bit(false));
            while rcc.cr.read().pllrdy().bit() {}

            rcc.cfgr.write(|w| unsafe {
                w.pllmul()
                    .bits(pllmul_bits)
                    .plldiv()
                    .bits(plldiv_bits)
                    .pllsrc()
                    .bit(hse_en)
            });

            rcc.cr.write(|w| {
                w.pllon()
                    .set_bit()
                    .hsi16on()
                    .bit(!hse_en)
                    .hseon()
                    .bit(hse_en)
            });
            if hse_en {
                while !rcc.cr.read().hserdy().bit() {}
            } else {
                while !rcc.cr.read().hsi16rdyf().bit() {}
            }
            while rcc.cr.read().pllrdy().bit_is_clear() {}

            // SW: PLL selected as system clock
            rcc.cfgr.modify(|_, w| unsafe {
                w.ppre2()
                    .bits(ppre2_bits)
                    .ppre1()
                    .bits(ppre1_bits)
                    .hpre()
                    .bits(hpre_bits)
                    .sw()
                    .bits(0b11)
            });
        } else {
            rcc.cr
                .write(|w| w.hsi16on().bit(!hse_en).hseon().bit(hse_en));

            if hse_en {
                while !rcc.cr.read().hserdy().bit() {}
            } else {
                while !rcc.cr.read().hsi16rdyf().bit() {}
            }

            // SW: HSI selected as system clock
            rcc.cfgr.write(|w| unsafe {
                w.ppre2()
                    .bits(ppre2_bits)
                    .ppre1()
                    .bits(ppre1_bits)
                    .hpre()
                    .bits(hpre_bits)
                    .sw()
                    .bits(if hse_en { 0b10 } else { 0b01 })
            });
        }

        Clocks {
            hclk: Hertz(hclk),
            pclk1: Hertz(pclk1),
            pclk2: Hertz(pclk2),
            ppre1,
            ppre2,
            sysclk: Hertz(sysclk_freq),
        }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
pub struct Clocks {
    hclk: Hertz,
    pclk1: Hertz,
    pclk2: Hertz,
    ppre1: u8,
    ppre2: u8,
    sysclk: Hertz,
}

impl Clocks {
    /// Returns the frequency of the AHB
    pub fn hclk(&self) -> Hertz {
        self.hclk
    }

    /// Returns the frequency of the APB1
    pub fn pclk1(&self) -> Hertz {
        self.pclk1
    }

    /// Returns the frequency of the APB2
    pub fn pclk2(&self) -> Hertz {
        self.pclk2
    }

    pub(crate) fn ppre1(&self) -> u8 {
        self.ppre1
    }

    pub(crate) fn ppre2(&self) -> u8 {
        self.ppre2
    }

    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
