/*
 * MIT License
 *
 * Copyright (c) 2020 Richard Healy
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
use core::ops;
use register::register_bitfields;
use register::mmio::ReadWrite;
use crate::debug; 

register_bitfields! {
    u32,
    BASIC_PENDING [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],
    PENDING_1 [
          UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],
    PENDING_2 [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],
    FIQ [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []    
    ],
    ENABLE_1 [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],
    ENABLE_2 [
        PCM OFFSET(55) NUMBITS(1) []
    ],
    ENABLE_BASIC [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],
    DISABLE_1 [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],
    DISABLE_2 [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],
    DISABLE_BASIC [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ]
}

///
///IRQ registers. 0x7E00B000
///
const IRQ_OFFSET:   u32 = 0x0000_B000;
pub const IRQ_BASE: u32 = MMIO_BASE + IRQ_OFFSET + 0x200;

///
///Puts DMA channels 0-14, interrupt status and enable registers in one
///structure.
///
#[repr(C)]
#[allow(non_snake_case)]
pub struct RegisterBlockIRQ {
    pub BASIC_PENDING: ReadWrite<u32, BASIC_PENDING::Register>,
    pub PENDING_1:     ReadWrite<u32, PENDING_1::Register>,
    pub PENDING_2:     ReadWrite<u32, PENDING_2::Register>,
    pub FIQ:           ReadWrite<u32, FIQ::Register>,
    pub ENABLE_1:      ReadWrite<u32, ENABLE_1::Register>,
    pub ENABLE_2:      ReadWrite<u32, ENABLE_2::Register>,
    pub ENABLE_BASIC:  ReadWrite<u32, ENABLE_BASIC::Register>,
    pub DISABLE_1:     ReadWrite<u32, DISABLE_1::Register>,
    pub DISABLE_2:     ReadWrite<u32, DISABLE_2::Register>,
    pub DISABLE_BASIC: ReadWrite<u32, DISABLE_BASIC::Register>,
}

///
/// IRQ peripheral registers
///
#[derive(Default)]
pub struct IRQ;

impl ops::Deref for IRQ {
    type Target = RegisterBlockIRQ;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl IRQ {
    #[inline]
    fn ptr() -> *const RegisterBlockIRQ {
        IRQ_BASE as *const _
    }
}
