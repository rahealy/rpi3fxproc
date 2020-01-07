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

#![no_std]

///
///Start address of the I/O peripherals mapped to the physical memory by
///hardware. If the MMU is set up to map this range to another base then
///software should use the MMU mapping.
///
pub const MMIO_BASE: u32 = 0x3F000000;

///
///Start address of the I/O peripherals mapped to the VC CPU bus. This
///base should be used when setting up the DMA controller.
///
pub const VCIO_BASE: u32 = 0x7E000000;

pub mod clk;
pub mod debug;
pub mod dma;
pub mod gpfsel;
pub mod gplev;
pub mod gpevt;
pub mod gpclr;
pub mod gpset;
pub mod i2c;
pub mod i2s;
pub mod irq;
pub mod mbox;
pub mod pwm;
pub mod timer;
pub mod uart;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
