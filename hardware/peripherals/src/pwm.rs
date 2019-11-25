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

//FIXME: Not done yet!

use super::MMIO_BASE;
use register::register_bitfields;
use register::mmio::ReadWrite;

/**********************************************************************
 * Clock Manager
 *
 * Reference
 *  https://www.scribd.com/doc/127599939/BCM2835-Audio-clocks
 * 
 *********************************************************************/

register_bitfields! {
    u32,

///Pulse width modulation control. 0x7E10_10A0
    CM_PWMCTL [
///Enter password before changing a value.
        PASSWD OFFSET(24) NUMBITS(8) [
            VAL = 0x5A
        ],

///MASH control
        MASH OFFSET(9) NUMBITS(2) [
            INT   = 0b00,   //Integer division
            ONE   = 0b01,   //One stage MASH
            TWO   = 0b10,   //Two stage MASH
            THREE = 0b11    //Three stage MASH
        ],

///Debug only. Generates edge on clock output.
        FLIP OFFSET(8) NUMBITS(1) [],

///Clock generator running.
        BUSY OFFSET(7) NUMBITS(1) [],

///Kill and restart clock generator. Debug only.
        KILL OFFSET(5) NUMBITS(1) [],

///Enable clock generator. Poll BUSY bit for result.
        ENAB OFFSET(4) NUMBITS(1) [],

///Clock source.
        SRC OFFSET(0) NUMBITS(4) [
            GND        = 0b0000,    //Ground (no clock)
            OSC        = 0b0001,    //Oscillator
            TESTDEBUG0 = 0b0010,    //???
            TESTDEBUG1 = 0b0011,    //???
            PLLA       = 0b0100,    //Phase locked loop A
            PLLC       = 0b0101,    //Phase locked loop C
            PLLD       = 0b0110,    //Phase locked loop D
            HDMIAUX    = 0b0111     //HDMI Auxillary
        ]
    ],

///PWM clock divider. 0x7E1010A4
    CM_PWMDIV [
///Enter password before changing a value.
        PASSWD OFFSET(24) NUMBITS(8) [
            VAL = 0x5A
        ],

///Integer part of divisor.
        DIVI OFFSET(12) NUMBITS(12) [],

///Fractional part of divisor.
        DIVF OFFSET(0) NUMBITS(12) []
    ]
}


///
///PWM clock control. 0x7E10_10A0
///
const CM_PWMCTL_OFFSET: u32 = 0x0010_10A0;
const CM_PWMCTL_BASE:   u32 = MMIO_BASE + CM_PWMCTL_OFFSET; 

///
///PWM clock control register.
///
pub const CM_PWMCTL: *const ReadWrite<u32, CM_PWMCTL::Register> =
    CM_PWMCTL_BASE as *const ReadWrite<u32, CM_PWMCTL::Register>;

///
///PWM clock divider. 0x7E10_10A4
///
const CM_PWMDIV_OFFSET:  u32 = 0x0010_10A4;
const CM_PWMDIV_BASE:    u32 = MMIO_BASE + CM_PWMDIV_OFFSET; 

///
///PWM clock divider register.
///
pub const CM_PWMDIV: *const ReadWrite<u32, CM_PWMDIV::Register> =
    CM_PWMDIV_BASE as *const ReadWrite<u32, CM_PWMDIV::Register>;
