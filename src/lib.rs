//! HAL for the STM32L0X3 family of microcontrollers

#![no_std]

pub use stm32l0x3;

pub mod exti;
pub mod flash;
pub mod gpio;
pub mod i2c;
pub mod lpusart;
pub mod prelude;
pub mod rcc;
pub mod time;
