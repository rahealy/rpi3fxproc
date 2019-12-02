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
 * GPCLR
 *********************************************************************/

register_bitfields! {
    u32,

///GPIO Clear pin.
    GPCLR0 [
///Clear RPi Pin 29 (BCM5) to put ultra2 into reset condition.
        PCLR5 OFFSET(5) NUMBITS(1) []
    ]
}

///
///GPCLR0 pin set register - 0x7E200028
///
const GPCLR0_OFFSET: u32 = 0x0020_0028;
const GPCLR0_BASE:   u32 = MMIO_BASE + GPCLR0_OFFSET;


///
///Register block representing all the GPCLR registers.
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockGPCLR {
    pub GPCLR0: ReadWrite<u32, GPCLR0::Register>
}

///
///Implements accessors to the GPCLR registers. 
///
#[derive(Default)]
pub struct GPCLR;

impl ops::Deref for GPCLR {
    type Target = RegisterBlockGPCLR;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl GPCLR {
    pub fn ptr() -> *const RegisterBlockGPCLR {
        GPCLR0_BASE as *const _
    }
}
