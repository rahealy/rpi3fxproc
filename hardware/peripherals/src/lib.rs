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
#![feature(asm)]

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
pub const VCIO_BASE: u32 = 0x7E00_0000;


///
///Flushes num 64bit blocks of memory from the cache starting at base.
///
#[inline]
pub fn flush_dcache_range(base: u64, num: u64) {
    for i in 0..num {
        unsafe { 
            asm!("dc civac, $0" :: "r"(base + ((i * 8) as u64)) :: "volatile"); 
        }
    }
}

///
///DMA uses VC CPU Bus addresses to access memory.
///
#[inline]
pub fn phy_mem_to_vc_loc(loc: u32) -> u32 {
//Mask off lower 24 bits and add base.
    (loc & 0x00FF_FFFF) + VCIO_BASE
}

///
///DMA uses VC CPU Bus addresses to access peripheral I/0.
///
#[inline]
pub fn mmio_to_vc_loc(loc: u32) -> u32 {
//Mask off lower 24 bits and add base.
    (loc - MMIO_BASE) + VCIO_BASE
}

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
