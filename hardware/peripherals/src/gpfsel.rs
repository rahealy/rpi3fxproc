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
 * GPFSEL
 *
 * I2C0 - Don't use I2C0 on RPi3!
 *  RPi I/O Pin 27, BCM GPIO0, Function: SDA, I2C: SDA0
 *  RPi I/O Pin 28, BCM GPIO1, Function: SCL, I2C: SCL0
 *
 * I2C1
 *  RPi I/O Pin  3, BCM GPIO2, Function: SDA, I2C: SDA1
 *  RPi I/O Pin  5, BCM GPIO3, Function: SCL, I2C: SCL1
 *
 * I2S Pins
 *  RPi I/O Pin 12, BCM GPIO18, Function: PCM_CLK, I2S: BCLK
 *  RPi I/O Pin 35, BCM GPIO19, Function: PCM_FS, I2S: LRCLK
 *  RPi I/O Pin 38, BCM GPIO20, Function: PCM_DIN, I2S: SDIN
 *  RPi I/O Pin 40, BCM GPIO21, Function: PCM_DOUT, I2S: SDOUT
 * 
 *********************************************************************/

register_bitfields! {
    u32,

/// GPIO Function Select 0
    GPFSEL0 [
/// I/O Pin 2 (SDA1)
        FSEL2 OFFSET(6) NUMBITS(3) [
            INPUT = 0b000,
            SDA1 = 0b100 // I2C1 SDA1 - Alternate function 0
        ],

/// I/O Pin 3 (SCL1)
        FSEL3 OFFSET(9) NUMBITS(3) [
            INPUT = 0b000,
            SCL1 = 0b100 // I2C1 SCL1 - Alternate function 0
        ],

/// RPi I/O Pin 29 (BCM5) used as reset for the ultra2.
        FSEL5 OFFSET(15) NUMBITS(3) [
            INPUT  = 0b000,
            OUTPUT = 0b001
        ]
    ],

/// GPIO Function Select 1
    GPFSEL1 [
/// I/O Pin 15 (RXD)
        FSEL15 OFFSET(15) NUMBITS(3) [
            INPUT = 0b000,
            RXD0 = 0b100, // UART0 (PL011) - Alternate function 0
            RXD1 = 0b010  // UART1 (Mini UART) - Alternate function 5

        ],

/// I/O Pin 14 (TXD)
        FSEL14 OFFSET(12) NUMBITS(3) [
            INPUT = 0b000,
            TXD0 = 0b100, // UART0 (PL011) - Alternate function 0
            TXD1 = 0b010  // UART1 (Mini UART) - Alternate function 5
        ],

/// I/O Pin 12 (BCLK)
        FSEL18 OFFSET(24) NUMBITS(3) [
            INPUT = 0b000,
            PCM_CLK = 0b100 // I2S - Alternate function 0
        ],

/// I/O Pin 35 (LRCLK)
        FSEL19 OFFSET(27) NUMBITS(3) [
            INPUT = 0b000,
            PCM_FS = 0b100 // I2S - Alternate function 0
        ]
    ],

/// GPIO Function Select 2
    GPFSEL2 [
/// I/O Pin 38 (SDIN)
        FSEL20 OFFSET(0) NUMBITS(3) [
            INPUT = 0b000,
            PCM_DIN = 0b100 // I2S - Alternate function 0
        ],

/// I/O Pin 40 (SDOUT)
        FSEL21 OFFSET(3) NUMBITS(3) [
            INPUT = 0b000,
            OUTPUT = 0b001,
            PCM_DOUT = 0b100 // I2S - Alternate function 0
        ],
        
///I/O Pin (ARM_TRST)
        FSEL22 OFFSET(6) NUMBITS(3) [
            INPUT = 0b000,
            OUTPUT = 0b001,
            ARM_TRST = 0b011 // JTAG - Alternate function 4
        ],
        
///I/O Pin (ARM_RTCK)
        FSEL23 OFFSET(9) NUMBITS(3) [
            INPUT = 0b000,
            OUTPUT = 0b001,
            ARM_RTCK = 0b011 // JTAG - Alternate function 4
        ],

///I/O Pin (ARM_TDO)
        FSEL24 OFFSET(12) NUMBITS(3) [
            INPUT = 0b000,
            OUTPUT = 0b001,
            ARM_TDO = 0b011 // JTAG - Alternate function 4
        ],

///I/O Pin (ARM_TCK)
        FSEL25 OFFSET(15) NUMBITS(3) [
            INPUT = 0b000,
            OUTPUT = 0b001,
            ARM_TCK = 0b011 // JTAG - Alternate function 4
        ],

///I/O Pin (ARM_TDI)
        FSEL26 OFFSET(18) NUMBITS(3) [
            INPUT = 0b000,
            OUTPUT = 0b001,
            ARM_TDI = 0b011 // JTAG - Alternate function 4
        ],

///I/O Pin (ARM_TMS)
        FSEL27 OFFSET(21) NUMBITS(3) [
            INPUT = 0b000,
            OUTPUT = 0b001,
            ARM_TMS = 0b011 // JTAG - Alternate function 4
        ]
    ]
}


///
///GPFSEL0 alternative function select register - 0x7E200000
///
const GPFSEL0_OFFSET: u32 = 0x0020_0000;
const GPFSEL0_BASE:   u32 = MMIO_BASE + GPFSEL0_OFFSET;


///
///Register block representing all the GPFSEL registers.
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockGPFSEL {
    pub GPFSEL0: ReadWrite<u32, GPFSEL0::Register>, // 0x00200000
    pub GPFSEL1: ReadWrite<u32, GPFSEL1::Register>, // 0x00200004
    pub GPFSEL2: ReadWrite<u32, GPFSEL2::Register>  // 0x00200008
}

///
///Implements accessors to the GPFSEL registers. 
///
#[derive(Default)]
pub struct GPFSEL;

impl ops::Deref for GPFSEL {
    type Target = RegisterBlockGPFSEL;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl GPFSEL {
    fn ptr() -> *const RegisterBlockGPFSEL {
        GPFSEL0_BASE as *const _
    }

///
///Select alternate GPIO pin functions for the I2C1 peripheral.
///
    pub fn fsel_i2c1(&self) {
        self.GPFSEL0.modify(GPFSEL0::FSEL2::INPUT + 
                            GPFSEL0::FSEL3::INPUT);

        self.GPFSEL0.modify(GPFSEL0::FSEL2::SDA1 + 
                            GPFSEL0::FSEL3::SCL1);
    }

///
///Select GPIO pin functions for the Ultra2 hat.
///
    pub fn fsel_ultra2(&self) {
        self.GPFSEL0.modify(GPFSEL0::FSEL5::OUTPUT);
    }

///
///Select alternate GPIO pin functions for the I2S peripheral.
///
    pub fn fsel_i2s(&self) {
        self.GPFSEL1.modify(GPFSEL1::FSEL18::INPUT);
        self.GPFSEL1.modify(GPFSEL1::FSEL18::PCM_CLK);
        
        self.GPFSEL1.modify(GPFSEL1::FSEL19::INPUT);
        self.GPFSEL1.modify(GPFSEL1::FSEL19::PCM_FS);

        self.GPFSEL2.modify(GPFSEL2::FSEL20::INPUT); 
        self.GPFSEL2.modify(GPFSEL2::FSEL20::PCM_DIN);

        self.GPFSEL2.modify(GPFSEL2::FSEL21::INPUT);
        self.GPFSEL2.modify(GPFSEL2::FSEL21::PCM_DOUT);
    }

    pub fn fsel_uart0(&self) {
        self.GPFSEL1.modify(GPFSEL1::FSEL14::INPUT + 
                            GPFSEL1::FSEL15::INPUT);
        self.GPFSEL1.modify(GPFSEL1::FSEL14::TXD0 + 
                            GPFSEL1::FSEL15::RXD0);
    }

    pub fn fsel_uart1(&self) {
        self.GPFSEL1.modify(GPFSEL1::FSEL14::INPUT + 
                            GPFSEL1::FSEL15::INPUT);
        self.GPFSEL1.modify(GPFSEL1::FSEL14::TXD1 + 
                            GPFSEL1::FSEL15::RXD1);
    }

    pub fn fsel_jtag(&self) {
        self.GPFSEL2.modify(GPFSEL2::FSEL22::INPUT);
        self.GPFSEL2.modify(GPFSEL2::FSEL22::ARM_TRST);

        self.GPFSEL2.modify(GPFSEL2::FSEL23::INPUT);
        self.GPFSEL2.modify(GPFSEL2::FSEL23::ARM_RTCK);

        self.GPFSEL2.modify(GPFSEL2::FSEL24::INPUT);
        self.GPFSEL2.modify(GPFSEL2::FSEL24::ARM_TDO);

        self.GPFSEL2.modify(GPFSEL2::FSEL25::INPUT);
        self.GPFSEL2.modify(GPFSEL2::FSEL25::ARM_TCK);

        self.GPFSEL2.modify(GPFSEL2::FSEL26::INPUT);
        self.GPFSEL2.modify(GPFSEL2::FSEL26::ARM_TDI);

        self.GPFSEL2.modify(GPFSEL2::FSEL27::INPUT);
        self.GPFSEL2.modify(GPFSEL2::FSEL27::ARM_TMS);
    }
}
