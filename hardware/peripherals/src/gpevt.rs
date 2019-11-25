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
 * GPIO Event registers. Sets GPIOs to trigger events on pin state transitions.
 */

use super::MMIO_BASE;
use register::register_bitfields;
use register::mmio::ReadWrite;
use core::ops;

register_bitfields! {
    u32,

///Event status detect. Write 1 to clear.
    GPEDS0 [
        EDS0 OFFSET(0) NUMBITS(1) [],
        EDS18 OFFSET(18) NUMBITS(1) [],
        EDS19 OFFSET(19) NUMBITS(1) [],
        EDS20 OFFSET(20) NUMBITS(1) [],
        EDS21 OFFSET(21) NUMBITS(1) []
    ],

    GPEDS1 [
        EDS32 OFFSET(0) NUMBITS(1) []
    ],

///Synchronous rising edge enable.
    GPREN0 [
        REN0 OFFSET(0) NUMBITS(1) [],
        REN18 OFFSET(18) NUMBITS(1) [],
        REN19 OFFSET(19) NUMBITS(1) [],
        REN20 OFFSET(20) NUMBITS(1) [],
        REN21 OFFSET(21) NUMBITS(1) []
    ],

    GPREN1 [
        REN32 OFFSET(0) NUMBITS(1) []
    ],

///Sychronous falling edge event enable.
    GPFEN0 [
        FEN0 OFFSET(0) NUMBITS(1) [],
        FEN18 OFFSET(18) NUMBITS(1) [],
        FEN19 OFFSET(19) NUMBITS(1) [],
        FEN20 OFFSET(20) NUMBITS(1) [],
        FEN21 OFFSET(21) NUMBITS(1) []
    ],

    GPFEN1 [
        FEN32 OFFSET(0) NUMBITS(1) []
    ],

///High detect enable.
    GPHEN0 [
        HEN0 OFFSET(0) NUMBITS(1) [],
        HEN18 OFFSET(18) NUMBITS(1) [],
        HEN19 OFFSET(19) NUMBITS(1) [],
        HEN20 OFFSET(20) NUMBITS(1) [],
        HEN21 OFFSET(21) NUMBITS(1) []
    ],

    GPHEN1 [
        HEN32 OFFSET(0) NUMBITS(1) []
    ],

///Low detect enable.
    GPLEN0 [
        LEN0 OFFSET(0) NUMBITS(1) [],
        LEN18 OFFSET(18) NUMBITS(1) [],
        LEN19 OFFSET(19) NUMBITS(1) [],
        LEN20 OFFSET(20) NUMBITS(1) [],
        LEN21 OFFSET(21) NUMBITS(1) []
    ],

    GPLEN1 [
        LEN32 OFFSET(0) NUMBITS(1) []
    ],

///Asyncronous rising edge event enable 
    GPAREN0 [
        AREN0 OFFSET(0) NUMBITS(1) [],
        AREN18 OFFSET(18) NUMBITS(1) [],
        AREN19 OFFSET(19) NUMBITS(1) [],
        AREN20 OFFSET(20) NUMBITS(1) [],
        AREN21 OFFSET(21) NUMBITS(1) []
    ],

    GPAREN1 [
        AREN32 OFFSET(0) NUMBITS(1) []
    ],

///Asyncronous falling edge event enable.
    GPAFEN0 [
        AFEN0 OFFSET(0) NUMBITS(1) [],
        AFEN18 OFFSET(18) NUMBITS(1) [],
        AFEN19 OFFSET(19) NUMBITS(1) [],
        AFEN20 OFFSET(20) NUMBITS(1) [],
        AFEN21 OFFSET(21) NUMBITS(1) []
    ],

    GPAFEN1 [
        AFEN32 OFFSET(0) NUMBITS(1) []
    ]
}

///
///GPEVT registers start - 0x7E200040
///
const GPEVT_OFFSET: u32 = 0x0020_0040;
const GPEVT_BASE:   u32 = MMIO_BASE + GPEVT_OFFSET;

///
///Register block representing all the GPEVT registers.
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockGPEVT {
    pub GPEDS0:  ReadWrite<u32, GPEDS0::Register>,
    pub GPEDS1:  ReadWrite<u32, GPEDS1::Register>,
    __res0:  u32,
    pub GPREN0:  ReadWrite<u32, GPREN0::Register>,
    pub GPREN1:  ReadWrite<u32, GPREN1::Register>,
    __res1:  u32,
    pub GPFEN0:  ReadWrite<u32, GPFEN0::Register>,
    pub GPFEN1:  ReadWrite<u32, GPFEN1::Register>,
    __res2:  u32,
    pub GPHEN0:  ReadWrite<u32, GPHEN0::Register>,
    pub GPHEN1:  ReadWrite<u32, GPHEN1::Register>,
    __res3:  u32,
    pub GPLEN0:  ReadWrite<u32, GPLEN0::Register>,
    pub GPLEN1:  ReadWrite<u32, GPLEN1::Register>,
    __res4:  u32,
    pub GPAREN0: ReadWrite<u32, GPAREN0::Register>,
    pub GPAREN1: ReadWrite<u32, GPAREN1::Register>,
    __res5:  u32,
    pub GPAFEN0: ReadWrite<u32, GPAFEN0::Register>,
    pub GPAFEN1: ReadWrite<u32, GPAFEN1::Register>
}

///
///Implements accessors to the GPEVT registers.
///
#[derive(Default)]
pub struct GPEVT;

impl ops::Deref for GPEVT {
    type Target = RegisterBlockGPEVT;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl GPEVT {
    fn ptr() -> *const RegisterBlockGPEVT {
        GPEVT_BASE as *const _
    }
}
