//! Inter-Integrated Circuit (I2C) bus

use stm32l0x3::{I2C1, I2C3};

use crate::gpio::gpioa::{PA10, PA9};
use crate::gpio::gpiob::{PB6, PB7, PB8, PB9};
use crate::gpio::{AF1, AF4, AF6};
use crate::rcc::{Clocks, APB1};
use crate::time::Hertz;
use core::cmp;
use embedded_hal::blocking::i2c::{Write, WriteRead};

/// I2C error
#[derive(Debug)]
pub enum Error {
    /// Bus error
    Bus,
    /// Arbitration loss
    Arbitration,
    // Overrun, // slave mode only
    // Pec, // SMBUS mode only
    // Timeout, // SMBUS mode only
    // Alert, // SMBUS mode only
    #[doc(hidden)]
    _Extensible,
}

// FIXME these should be "closed" traits
/// SCL pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SclPin<I2C> {}

/// SDA pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SdaPin<I2C> {}

unsafe impl SclPin<I2C1> for PA9<AF6> {}
unsafe impl SclPin<I2C1> for PB6<AF1> {}
unsafe impl SclPin<I2C1> for PB8<AF4> {}

unsafe impl SdaPin<I2C1> for PA10<AF6> {}
unsafe impl SdaPin<I2C1> for PB7<AF1> {}
unsafe impl SdaPin<I2C1> for PB9<AF4> {}

/// I2C peripheral operating in master mode
pub struct I2c<I2C, PINS> {
    i2c: I2C,
    pins: PINS,
}

macro_rules! busy_wait {
    ($i2c:expr, $flag:ident) => {
        loop {
            let isr = $i2c.isr.read();

            if isr.berr().bit_is_set() {
                return Err(Error::Bus);
            } else if isr.arlo().bit_is_set() {
                return Err(Error::Arbitration);
            } else if isr.$flag().bit_is_set() {
                break;
            } else {
                // try again
            }
        }
    };
}

macro_rules! hal {
    ($($I2CX:ident: ($i2cX:ident, $i2cXen:ident, $i2cXrst:ident),)+) => {
        $(
            impl<SCL, SDA> I2c<$I2CX, (SCL, SDA)> {
                /// Configures the I2C peripheral to work in master mode
                pub fn $i2cX<F>(
                    i2c: $I2CX,
                    pins: (SCL, SDA),
                    freq: F,
                    clocks: &Clocks,
                    apb1: &mut APB1,
                ) -> Self where
                    F: Into<Hertz>,
                    SCL: SclPin<$I2CX>,
                    SDA: SdaPin<$I2CX>,
                {
                    apb1.enr().modify(|_, w| w.$i2cXen().set_bit());
                    apb1.rstr().modify(|_, w| w.$i2cXrst().set_bit());
                    apb1.rstr().modify(|_, w| w.$i2cXrst().clear_bit());

                    let freq = freq.into().0;

                    assert!(freq <= 100_000);

                    // TODO review compliance with the timing requirements of I2C
                    // t_I2CCLK = 1 / PCLK1
                    // t_PRESC  = (PRESC + 1) * t_I2CCLK
                    // t_SCLL   = (SCLL + 1) * t_PRESC
                    // t_SCLH   = (SCLH + 1) * t_PRESC
                    //
                    // t_SYNC1 + t_SYNC2 > 4 * t_I2CCLK
                    // t_SCL ~= t_SYNC1 + t_SYNC2 + t_SCLL + t_SCLH
                    let i2cclk = clocks.pclk1().0;

                    // // standard-mode only
                    let presc = 1;
                    let scll = ((((i2cclk >> presc) >> 1) / freq) - 1) as u8;
                    let sclh = scll - 4;
                    let sdadel = 2;
                    let scldel = 4;

                    // Configure for "standard mode" (100 KHz)
                    i2c.timingr.write(|w| unsafe {
                        w.presc()
                            .bits(presc)
                            .scll()
                            .bits(scll)
                            .sclh()
                            .bits(sclh)
                            .sdadel()
                            .bits(sdadel)
                            .scldel()
                            .bits(scldel)
                    });

                    // Enable the peripheral
                    i2c.cr1.write(|w| w.pe().set_bit());

                    I2c { i2c, pins }
                }

                /// Releases the I2C peripheral and associated pins
                pub fn free(self) -> ($I2CX, (SCL, SDA)) {
                    (self.i2c, self.pins)
                }
            }

            impl<PINS> Write for I2c<$I2CX, PINS> {
                type Error = Error;

                fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
                    // TODO support transfers of more than 255 bytes
                    assert!(bytes.len() < 256 && bytes.len() > 0);

                    // TODO do we have to explicitly wait here if the bus is busy (e.g. another
                    // master is communicating)?

                    // START and prepare to send `bytes`
                    self.i2c.cr2.write(|w| unsafe {
                        w.sadd()
                            .bits(addr as u16)
                            .rd_wrn()
                            .clear_bit()
                            .nbytes()
                            .bits(bytes.len() as u8)
                            .start()
                            .set_bit()
                            .autoend()
                            .set_bit()
                    });

                    for byte in bytes.iter() {
                        // Wait until we are allowed to send data (START has been ACKed or last byte
                        // when through)
                        busy_wait!(self.i2c, txis);

                        // put byte on the wire
                        self.i2c.txdr.write(|w| unsafe {
                            w.txdata().bits(*byte)
                        });
                    }

                    // Wait until the last transmission is finished
                    //busy_wait!(self.i2c, tc);

                    // automatic STOP

                    Ok(())
                }
            }

            impl<PINS> WriteRead for I2c<$I2CX, PINS> {
                type Error = Error;

                fn write_read(
                    &mut self,
                    addr: u8,
                    bytes: &[u8],
                    buffer: &mut [u8],
                ) -> Result<(), Error> {
                    // TODO support transfers of more than 255 bytes
                    assert!(bytes.len() < 256 && bytes.len() > 0);
                    assert!(buffer.len() < 256 && buffer.len() > 0);

                    // TODO do we have to explicitly wait here if the bus is busy (e.g. another
                    // master is communicating)?

                    // START and prepare to send `bytes`
                    self.i2c.cr2.write(|w| unsafe {
                        w.sadd()
                            .bits(addr as u16)
                            .rd_wrn()
                            .clear_bit()
                            .nbytes()
                            .bits(bytes.len() as u8)
                            .start()
                            .set_bit()
                            .autoend()
                            .clear_bit()
                    });

                    for byte in bytes.iter() {
                        // Wait until we are allowed to send data (START has been ACKed or last byte
                        // when through)
                        busy_wait!(self.i2c, txis);

                        // put byte on the wire
                        self.i2c.txdr.write(|w| unsafe {
                            w.txdata().bits(*byte)
                        });
                    }

                    // Wait until the last transmission is finished
                    busy_wait!(self.i2c, tc);

                    // reSTART and prepare to receive bytes into `buffer`
                    self.i2c.cr2.write(|w| unsafe {
                        w.sadd()
                            .bits(addr as u16)
                            .rd_wrn()
                            .set_bit()
                            .nbytes()
                            .bits(buffer.len() as u8)
                            .start()
                            .set_bit()
                            .autoend()
                            .set_bit()
                    });

                    for byte in buffer.iter_mut() {
                        // Wait until we have received something
                        busy_wait!(self.i2c, rxne);

                        *byte = self.i2c.rxdr.read().rxdata().bits();
                    }

                    // automatic STOP

                    Ok(())
                }
            }
        )+
    }
}

hal! {
    I2C1: (i2c1, i2c1en, i2c1rst),
    I2C3: (i2c3, i2c3en, i2c3rst),
}
