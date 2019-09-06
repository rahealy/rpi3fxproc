/*
 * MIT License
 *
 * Copyright (c) 2019 Richard Healy
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */ 

/*
 * I2S Pins
 * RPi I/O Pin 12, BCM18, Function: PCM_CLK, I2S: BCLK
 * RPi I/O Pin 35, BCM19, Function: PCM_FS, I2S: LRCLK
 * RPi I/O Pin 38, BCM20, Function: PCM_DIN, I2S: SDIN
 * RPi I/O Pin 40, BCM21, Function: PCM_DOUT, I2S: SOUT
 */

use peripherals::MMIO_BASE;
use peripherals::{PERIPHERALS, i2c, timer};
use drivers::cs4265;
use register::register_bitfields;
use register::mmio::ReadWrite;

/**********************************************************************
 * GPIO
 *********************************************************************/

register_bitfields! {
    u32,

/// GPIO Function Select 0
    GPFSEL0 [
/// RPi I/O Pin 29 (BCM5) used as reset for the ultra2.
        FSEL5 OFFSET(15) NUMBITS(3) [
            OUTPUT = 0b001
        ]
    ],

///GPIO Set pin.
    GPSET0 [
///Set RPi I/O Pin 29 (BCM5) to bring ultra2 out of reset condition.
        PSET5 OFFSET(5) NUMBITS(1) []
    ],

///GPIO Clear pin.
    GPCLR0 [
///Clear RPi Pin 29 (BCM5) to put ultra2 into reset condition.
        PCLR5 OFFSET(5) NUMBITS(1) []
    ]
}


///
///GPFSEL2 alternative function select register - 0x7E200008
///
const GPFSEL0_OFFSET: u32 = 0x0020_0000;
const GPFSEL0_BASE:   u32 = MMIO_BASE + GPFSEL0_OFFSET;

///
///Function select register for the GPIO pin used by the Ultra2 board.
///
pub const GPFSEL0: *const ReadWrite<u32, GPFSEL0::Register> =
    GPFSEL0_BASE as *const ReadWrite<u32, GPFSEL0::Register>;


///
///GPSET0 pin set register - 0x7E20001C
///
const GPSET0_OFFSET: u32 = 0x0020_001C;
const GPSET0_BASE:   u32 = MMIO_BASE + GPSET0_OFFSET;

///
///Output set register for the GPIO pin used by the Ultra2 board.
///
pub const GPSET0: *const ReadWrite<u32, GPSET0::Register> =
    GPSET0_BASE as *const ReadWrite<u32, GPSET0::Register>;


///
///GPCLR0 pin set register - 0x7E200028
///
const GPCLR0_OFFSET: u32 = 0x0020_0028;
const GPCLR0_BASE:   u32 = MMIO_BASE + GPCLR0_OFFSET;

///
///Output clear register for the GPIO pin used by the Ultra2 board.
///
pub const GPCLR0: *const ReadWrite<u32, GPCLR0::Register> =
    GPCLR0_BASE as *const ReadWrite<u32, GPCLR0::Register>;


/**********************************************************************
 * Ultra2
 *********************************************************************/

pub struct Ultra2 {
    pub cs4265: cs4265::CS4265
}

pub enum ERROR {
    I2C(i2c::ERROR),
    Timer(timer::ERROR),
    CS4265(cs4265::ERROR)
}
 
impl ERROR {
    pub fn msg (&self) -> &'static str {
        match self {
            ERROR::I2C(err) => err.msg(),
            ERROR::Timer(err) => err.msg(),
            ERROR::CS4265(err) => err.msg()
        }
    }
}

impl Ultra2 {
    pub fn new() -> Ultra2 {
        Ultra2 {
            cs4265: cs4265::CS4265::new()
        }
    }

///
/// Bring ultra2 out of reset. Poll for condition of CS4265 SDOUT pin 
/// and save i2c address for use in further accesses.
///
    pub fn init(&mut self) -> Result<(), ERROR> {
        PERIPHERALS.uart.puts("ultra2.init(): Releasing reset. Waiting two seconds for settle.\r\n");
        unsafe {
            (*GPFSEL0).modify(GPFSEL0::FSEL5::OUTPUT);
            (*GPSET0).modify(GPSET0::PSET5::SET);
        }

        if let Err(err) = PERIPHERALS.timer.one_shot(1, 2_000_000) {
            return Err(ERROR::Timer(err));
        }

        PERIPHERALS.uart.puts("ultra2.init(): Initializing cs4265.\r\n");
        if let Err(err) = self.cs4265.init() {
            match err {
                cs4265::ERROR::I2C(e) => {
                    return Err(ERROR::I2C(e));
                },
                _ => {
                    return Err(ERROR::CS4265(err));
                }
            }
        }
        
        return Ok(());
    }
}
