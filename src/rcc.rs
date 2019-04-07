//! Reset and Clock Control

use stm32l0x3::{rcc, RCC};

/// Extension trait that constrains the `RCC` peripheral
pub trait RccExt {
    /// Constrains the `RCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Rcc;
}

impl RccExt for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            gpio: GPIO { _0: () },
        }
    }
}

/// RCC peripheral
pub struct Rcc {
    pub gpio: GPIO,
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
