//! HAL for the STM32L0X3 family of microcontrollers

#![no_std]

pub use stm32l0x3;

pub mod rcc;
pub mod gpio;
pub mod prelude;
