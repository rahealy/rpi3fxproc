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

//
//Manages clocks.
//

use super::MMIO_BASE;
use core::ops;
use register::register_bitfields;
use register::mmio::ReadWrite;
use crate::debug;

/**********************************************************************
 * PCMDIV
 *
 * Reference
 *  https://www.scribd.com/doc/127599939/BCM2835-Audio-clocks
 *********************************************************************/

register_bitfields! {
    u32,

///PCM clock divider. 0x7E10_109C
    CM_PCMDIV [
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
///PCM clock divider. 0x7E10_109C
///
const CM_PCMDIV_OFFSET: u32 = 0x0010_109C;
const CM_PCMDIV_BASE:   u32 = MMIO_BASE + CM_PCMDIV_OFFSET; 

///Clock divder.
#[derive(Default)]
struct PCMDIV;

impl ops::Deref for PCMDIV {
    type Target = ReadWrite<u32, CM_PCMDIV::Register>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl PCMDIV {

    fn ptr() -> *const ReadWrite<u32, CM_PCMDIV::Register> {
        CM_PCMDIV_BASE as *const _
    }

///
///Reference
/// https://github.com/arisena-com/rpi_src/blob/master/apps/i2s_test/src/i2s_test.c
///
/// Set 12 bit integer and fractional values of divider.
///
    fn set(&self, fsck: u32) {
        self.write(CM_PCMDIV::PASSWD::VAL);
        self.write (
            CM_PCMDIV::PASSWD::VAL +
            CM_PCMDIV::DIVI.val(19_200_000 / fsck) +                 //Integer divisor of 10
            CM_PCMDIV::DIVF.val((4095 * (19_200_000 % fsck)) / fsck) //Fractional divisor of 1/4095
        );
    }
    
    fn clear(&self) {
        self.write(CM_PCMDIV::PASSWD::VAL);
        self.write (
            CM_PCMDIV::PASSWD::VAL +
            CM_PCMDIV::DIVI.val(0) +                 //Integer divisor of 10
            CM_PCMDIV::DIVF.val(0) //Fractional divisor of 1/4095
        );
    }
}

/**********************************************************************
 * PCMCTL
 *
 * Reference
 *  https://www.scribd.com/doc/127599939/BCM2835-Audio-clocks
 *********************************************************************/

register_bitfields! {
    u32,

///PCM clock control. 0x7E10_1098
    CM_PCMCTL [
///Enter password before changing a value.
        PASSWD OFFSET(24) NUMBITS(8) [
            VAL = 0x5A
        ],

///MASH control
        MASH OFFSET(9) NUMBITS(2) [
            INT   = 0b00,   //Integer division, ignores fractional part of divider.
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
            OSC        = 0b0001,    //19.2 MHz oscillator
            TESTDEBUG0 = 0b0010,    //???
            TESTDEBUG1 = 0b0011,    //???
            PLLA       = 0b0100,    //Phase locked loop A
            PLLC       = 0b0101,    //Phase locked loop C
            PLLD       = 0b0110,    //Phase locked loop D
            HDMIAUX    = 0b0111     //HDMI Auxillary
        ]
    ]
}


///
///PCM (I2C) clock control. 0x7E10_1098
///
const CM_PCMCTL_OFFSET: u32 = 0x0010_1098;
const CM_PCMCTL_BASE:   u32 = MMIO_BASE + CM_PCMCTL_OFFSET; 


///Clock control.
#[derive(Default)]
pub struct PCMCTL;

impl ops::Deref for PCMCTL {
    type Target = ReadWrite<u32, CM_PCMCTL::Register>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl PCMCTL {

    fn ptr() -> *const ReadWrite<u32, CM_PCMCTL::Register> {
        CM_PCMCTL_BASE as *const _
    }

    fn wait_busy(&self, val: bool) {
        while self.is_set(CM_PCMCTL::BUSY) != val {}
    }

///
/// I2S SCK frequency 'fsck' = SampleRate x BitsPerChannel x numberOfChannels
///
///Reference
/// https://github.com/arisena-com/rpi_src/blob/master/apps/i2s_test/src/i2s_test.c
///
    pub fn i2s_enable(&self, fsck: u32) {
        debug::out("pcmctl.i2s_enable(): Setting up PCM clock for i2s operation.\r\n");

//Disable clock.
        debug::out("pcmctl.i2s_enable(): Disabling clock.\r\n");
        self.modify (
            CM_PCMCTL::PASSWD::VAL +
            CM_PCMCTL::ENAB::CLEAR
        );
        self.wait_busy(false);

//Set the PCM control registers.
        debug::out("pcmctl.i2s_enable(): Configuring clock.\r\n");
        self.modify (
            CM_PCMCTL::PASSWD::VAL +
            CM_PCMCTL::MASH::INT + //MASH set to integer.
            CM_PCMCTL::SRC::OSC    //Use oscillator for clock source.
        );

//Oscillator clock source is fixed at 19200000Hz.
        debug::out("pcmctl.i2s_enable(): Setting clock divider.\r\n");
        PCMDIV::default().set(fsck);

//Keep the control values used to set divider and enable. Wait until started.
        debug::out("pcmctl.i2s_enable(): Enabling clock.\r\n");
        self.modify (
            CM_PCMCTL::PASSWD::VAL +
            CM_PCMCTL::MASH::INT   + //MASH set to integer.
            CM_PCMCTL::SRC::OSC    + //Use oscillator for clock source.
            CM_PCMCTL::ENAB::SET
        );

        self.wait_busy(true);
        debug::out("pcmctl.i2s_enable(): PCM setup for i2s operation complete.\r\n");
    }

    pub fn i2s_disable(&self) {
        debug::out("pcmctl.i2s_disable(): Disabling clock.\r\n");
        self.modify (
            CM_PCMCTL::PASSWD::VAL +
            CM_PCMCTL::ENAB::CLEAR
        );
        self.wait_busy(false);

        debug::out("pcmctl.i2s_disable(): Clearing clock divider.\r\n");
        PCMDIV::default().clear();
        debug::out("pcmctl.i2s_disable(): PCM disable for i2s operation complete.\r\n");
    }

}
