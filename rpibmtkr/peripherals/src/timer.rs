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
use super::TIMER_MIN_RESOLUTION_MSECS;
use register::register_bitfields;
use register::mmio::ReadWrite;
use core::ops;

register_bitfields! {
    u32,

///
/// Control and status of timer. If Mn set then low 32 bits of system 
/// timer matched 32 bit value in Cn. Clear bit to reset. Offset 0x0
///
    CS [
        M3 OFFSET(3) NUMBITS(1) [],
        M2 OFFSET(2) NUMBITS(1) [],
        M1 OFFSET(1) NUMBITS(1) [],
        M0 OFFSET(0) NUMBITS(1) []
    ],

///Timer counter low 32 bits. Offset 0x4.
    CLO [
        CNT OFFSET(0) NUMBITS(32) []
    ],
    
///Timer counter high 32 bits. Offset 0x8.
    CHI [
        CNT OFFSET(0) NUMBITS(32) []
    ],
///Timer compare 0. Reserved by GPU. Offset 0xC.
    C0 [
        CMP OFFSET(0) NUMBITS(32) []
    ],
///Timer compare 1. Offset 0x10.
    C1 [
        CMP OFFSET(0) NUMBITS(32) []
    ],
///Timer compare 2. Reserved by GPU. Offset 0x14.
    C2 [
        CMP OFFSET(0) NUMBITS(32) []
    ],
///Timer compare 3. Offset 0x18.
    C3 [
        CMP OFFSET(0) NUMBITS(32) []
    ]
}

///
/// Timer peripheral registers
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
///Control and status.
    CS:     ReadWrite<u32, CS::Register>,
///Timer counter low 32 bits.
    CLO:    ReadWrite<u32, CLO::Register>,
///Timer counter high 32 bits.
    CHI:    ReadWrite<u32, CHI::Register>,
///Timer compare 0.
    C0:     ReadWrite<u32, C0::Register>,
///Timer compare 1.
    C1:     ReadWrite<u32, C1::Register>,
///Timer compare 2.
    C2:     ReadWrite<u32, C2::Register>,
///Timer compare 3.
    C3:     ReadWrite<u32, C3::Register>
}

pub enum ERROR {
    RESOLUTION,
    INVALID,
    RESERVED
}

impl ERROR {
    pub fn msg (&self) -> &'static str {
        match self {
            ERROR::RESOLUTION   => "Time value beneath useful timer resolution.",
            ERROR::INVALID      => "Timer doesn't exist.",
            ERROR::RESERVED     => "Timer reserved by GPU and not available."
        }
    }
}

///
///Timer offset. 0x7E003000.
///
const TIMER_OFFSET:  u32 = 0x0000_3000;
const TIMER_BASE:    u32 = MMIO_BASE + TIMER_OFFSET; 

pub struct Timer;

impl ops::Deref for Timer {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl Timer {
    fn ptr() -> *const RegisterBlock {
        TIMER_BASE as *const _
    }

    pub const fn new() -> Timer {
        Timer
    }

    pub fn init(&self) {
        
    }


///
/// Use a single timer t to delay a number of ticks. Returns after number of 
/// ticks have elapsed. Assumes no IRQ is set. Timer is 1MHz so tick is 1ms.
///
    pub fn one_shot(&self, t: u32, msecs: u32) -> Result<(), ERROR> {
        if msecs < TIMER_MIN_RESOLUTION_MSECS {
            return Err(ERROR::RESOLUTION);
        }

        let tval = self.CLO.get() + msecs;
 
        match t {
            0 => { return Err(ERROR::RESERVED); },

            1 => {
                self.C1.set(tval);
                self.CS.modify(CS::M1::SET);
                while !(self.CS.is_set(CS::M1)) {}
            },

            2 => { return Err(ERROR::RESERVED); },

            3 => { 
                self.C3.set(tval);
                self.CS.modify(CS::M3::SET);
                while !(self.CS.is_set(CS::M3)) {}
            },

            _ => { return Err(ERROR::INVALID); }
        }

        return Ok(());
    }
}
