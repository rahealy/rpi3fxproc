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

use crate::uart;
use crate::mbox;
use crate::i2c;
use crate::timer;

///
/// All system peripherals represented in a single structure.
///
#[repr(C)]
pub struct Peripherals {
    pub mbox: mbox::Mbox,
    pub uart: uart::Uart,
    pub i2c1: i2c::I2C1,
    pub timer: timer::Timer
}

impl Peripherals {
    pub const fn new() -> Peripherals {
        Peripherals {
            mbox: mbox::Mbox::new(),
            uart: uart::Uart::new(),
            i2c1: i2c::I2C1::new(),
            timer: timer::Timer::new()
        }
    }

    pub fn init(&mut self) {
        if self.uart.init(&mut self.mbox).is_err() {
            panic!();
        }
        for _ in 0..70 { self.uart.puts("."); }
        self.uart.puts("\r\n");
        self.uart.puts("UART Initialized.\r\n");
        self.i2c1.init();
        self.uart.puts("I2C Initialized.\r\n");
        self.timer.init();
        self.uart.puts("Timer Initialized.\r\n");
    }
}
