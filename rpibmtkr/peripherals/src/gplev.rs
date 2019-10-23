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
 * GPIO Level registers. Reads the actual level of the pin.
 */

use super::MMIO_BASE;
use register::register_bitfields;
use register::mmio::ReadOnly;
use core::ops;

register_bitfields! {
    u32,

///GPIO level bank 0. 0x7E200034
    GPLEV0 [
/// I/O Pin 12 (BCLK)
        LEV18 OFFSET(18) NUMBITS(1) [],

/// I/O Pin 35 (LRCLK)
        LEV19 OFFSET(19) NUMBITS(1) [],

/// I/O Pin 38 (SDIN)
        LEV20 OFFSET(20) NUMBITS(1) [],

/// I/O Pin 40 (SDOUT)
        LEV21 OFFSET(21) NUMBITS(1) []
    ],

///GPIO level bank 1. 0x7E200038
    GPLEV1 [
///LEV 0
        LEV32 OFFSET(0) NUMBITS(1) []
    ]
}

///
///GPIO pin level - 0x7E200034
///
const GPLEV0_OFFSET: u32 = 0x0020_0034;
const GPLEV0_BASE:   u32 = MMIO_BASE + GPLEV0_OFFSET;

///
///Register block representing all the GPLEV registers.
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockGPLEV {
    pub GPLEV0: ReadOnly<u32, GPLEV0::Register>, // 0x00200034
    pub GPLEV1: ReadOnly<u32, GPLEV1::Register>, // 0x00200038
}

///
///Implements accessors to the GPLEV registers.
///
#[derive(Default)]
pub struct GPLEV;

impl ops::Deref for GPLEV {
    type Target = RegisterBlockGPLEV;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl GPLEV {
    fn ptr() -> *const RegisterBlockGPLEV {
        GPLEV0_BASE as *const _
    }
}
