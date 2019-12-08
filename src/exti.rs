//! External interrupt and event controller

use crate::rcc;
use stm32l0x3::{exti, EXTI, SYSCFG_COMP};

/// Extension trait that constrains the `EXTI` peripheral
pub trait ExtiExt {
    /// Constrains the `EXTI` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Exti;
}

impl ExtiExt for EXTI {
    fn constrain(self) -> Exti {
        Exti {
            exti0: EXTI0 {},
            exti1: EXTI1 {},
            exti2: EXTI2 {},
            exti3: EXTI3 {},
            exti4: EXTI4 {},
            exti5: EXTI5 {},
            exti6: EXTI6 {},
            exti7: EXTI7 {},
            exti8: EXTI8 {},
            exti9: EXTI9 {},
            exti10: EXTI10 {},
            exti11: EXTI11 {},
            exti12: EXTI12 {},
            exti13: EXTI13 {},
            exti14: EXTI14 {},
            exti15: EXTI15 {},
        }
    }
}

/// Constrained EXTI peripheral
pub struct Exti {
    pub exti0: EXTI0,
    pub exti1: EXTI1,
    pub exti2: EXTI2,
    pub exti3: EXTI3,
    pub exti4: EXTI4,
    pub exti5: EXTI5,
    pub exti6: EXTI6,
    pub exti7: EXTI7,
    pub exti8: EXTI8,
    pub exti9: EXTI9,
    pub exti10: EXTI10,
    pub exti11: EXTI11,
    pub exti12: EXTI12,
    pub exti13: EXTI13,
    pub exti14: EXTI14,
    pub exti15: EXTI15,
}

pub enum GpioExtiSource {
    GPIOA,
    GPIOB,
    GPIOC,
    GPIOD,
    GPIOE,
    GPIOH,
}

impl GpioExtiSource {
    fn syscfg_bits(self) -> u8 {
        match self {
            GpioExtiSource::GPIOA => 0b0000,
            GpioExtiSource::GPIOB => 0b0001,
            GpioExtiSource::GPIOC => 0b0010,
            GpioExtiSource::GPIOD => 0b0011,
            GpioExtiSource::GPIOE => 0b0100,
            GpioExtiSource::GPIOH => 0b0101,
        }
    }
}

pub enum ExtiTrigger {
    Rising,
    Falling,
    RisingAndFalling,
}

pub trait GpioExti {
    fn configure_gpio_interrupt(
        &mut self,
        apb2: &mut rcc::APB2,
        syscfg: &mut SYSCFG_COMP,
        source: GpioExtiSource,
        trigger: ExtiTrigger,
    );

    fn mask(&mut self);
    fn unmask(&mut self);
    fn is_pending(&self) -> bool;
    fn clear_pending(&self);
}

macro_rules! exti_gpio_line {
    ($EXTIX:ident, $extix: ident, $SYSCFGR:ident, $imr:ident, $rtsr:ident, $ftsr:ident, $pif: ident) => {
        pub struct $EXTIX {}

        impl GpioExti for $EXTIX {
            fn configure_gpio_interrupt(
                &mut self,
                apb2: &mut rcc::APB2,
                syscfg: &mut SYSCFG_COMP,
                source: GpioExtiSource,
                trigger: ExtiTrigger,
            ) {
                apb2.enr().modify(|_, w| w.syscfgen().set_bit());
                syscfg
                    .$SYSCFGR
                    .modify(|_, w| unsafe { w.$extix().bits(source.syscfg_bits()) });
                unsafe {
                    (*EXTI::ptr()).imr.modify(|_, w| w.$imr().set_bit());
                }
                match trigger {
                    ExtiTrigger::Rising | ExtiTrigger::RisingAndFalling => unsafe {
                        (*EXTI::ptr()).rtsr.modify(|_, w| w.$rtsr().set_bit());
                    },
                    _ => unsafe {
                        (*EXTI::ptr()).rtsr.modify(|_, w| w.$rtsr().clear_bit());
                    },
                }
                match trigger {
                    ExtiTrigger::Falling | ExtiTrigger::RisingAndFalling => unsafe {
                        (*EXTI::ptr()).ftsr.modify(|_, w| w.$ftsr().set_bit());
                    },
                    _ => unsafe {
                        (*EXTI::ptr()).ftsr.modify(|_, w| w.$ftsr().clear_bit());
                    },
                }
            }

            fn mask(&mut self) {
                unsafe {
                    (*EXTI::ptr()).imr.modify(|_, w| w.$imr().clear_bit());
                }
            }

            fn unmask(&mut self) {
                unsafe {
                    (*EXTI::ptr()).imr.modify(|_, w| w.$imr().set_bit());
                }
            }

            fn is_pending(&self) -> bool {
                unsafe { (*EXTI::ptr()).pr.read().$pif().bit() }
            }

            fn clear_pending(&self) {
                unsafe {
                    (*EXTI::ptr()).pr.write(|w| w.$pif().set_bit());
                }
            }
        }
    };
}

exti_gpio_line!(EXTI0, exti0, exticr1, im0, rt0, ft0, pif0);
exti_gpio_line!(EXTI1, exti1, exticr1, im1, rt1, ft1, pif1);
exti_gpio_line!(EXTI2, exti2, exticr1, im2, rt2, ft2, pif2);
exti_gpio_line!(EXTI3, exti3, exticr1, im3, rt3, ft3, pif3);
exti_gpio_line!(EXTI4, exti4, exticr2, im4, rt4, ft4, pif4);
exti_gpio_line!(EXTI5, exti5, exticr2, im5, rt5, ft5, pif5);
exti_gpio_line!(EXTI6, exti6, exticr2, im6, rt6, ft6, pif6);
exti_gpio_line!(EXTI7, exti7, exticr2, im7, rt7, ft7, pif7);
exti_gpio_line!(EXTI8, exti8, exticr3, im8, rt8, ft8, pif8);
exti_gpio_line!(EXTI9, exti9, exticr3, im9, rt9, ft9, pif9);
exti_gpio_line!(EXTI10, exti10, exticr3, im10, rt10, ft10, pif10);
exti_gpio_line!(EXTI11, exti11, exticr3, im11, rt11, ft11, pif11);
exti_gpio_line!(EXTI12, exti12, exticr4, im12, rt12, ft12, pif12);
exti_gpio_line!(EXTI13, exti13, exticr4, im13, rt13, ft13, pif13);
exti_gpio_line!(EXTI14, exti14, exticr4, im14, rt14, ft14, pif14);
exti_gpio_line!(EXTI15, exti15, exticr4, im15, rt15, ft15, pif15);
