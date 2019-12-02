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


use super::MMIO_BASE;
use register::register_bitfields;
use register::mmio::ReadWrite;
use core::ops;

/**********************************************************************
 * GPSET
 *********************************************************************/

register_bitfields! {
    u32,

///GPIO Set pin.
    GPSET0 [
///Set RPi I/O Pin 29 (BCM5) to bring ultra2 out of reset condition.
        PSET5 OFFSET(5) NUMBITS(1) []
    ]
}

///
///GPSET0 pin set register - 0x7E20001C
///
const GPSET0_OFFSET: u32 = 0x0020_001C;
const GPSET0_BASE:   u32 = MMIO_BASE + GPSET0_OFFSET;

///
///Register block representing all the GPSET registers.
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockGPSET {
    pub GPSET0: ReadWrite<u32, GPSET0::Register>
}

///
///Implements accessors to the GPSET registers. 
///
#[derive(Default)]
pub struct GPSET;

impl ops::Deref for GPSET {
    type Target = RegisterBlockGPSET;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl GPSET {
    pub fn ptr() -> *const RegisterBlockGPSET {
        GPSET0_BASE as *const _
    }
}
 
