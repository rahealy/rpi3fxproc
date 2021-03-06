/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
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
use crate::mbox;
use crate::gpfsel::GPFSEL;
use core::ops;
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::asm;
use register::register_bitfields;
use register::mmio::ReadWrite;


/**********************************************************************
 * GPPUD
 *********************************************************************/

register_bitfields! {
    u32,

///GPIO Pull-up/down Register controls all the GPIO pins.
    GPPUD [
        PUD OFFSET(0) NUMBITS(2) [
            OFF   = 0b00,
            ENPD  = 0b01, //Enable pull down.
            ENPU  = 0b10  //Enable pull up.
        ]
    ]
}

///
///GPPUD GPIO pin clock enable - 0x7E200094
///
const GPPUD_OFFSET: u32 = 0x0020_0094;
const GPPUD_BASE:   u32 = MMIO_BASE + GPPUD_OFFSET;


///
/// GPPUD peripheral registers
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockGPPUD {
    GPPUD0: ReadWrite<u32, GPPUD::Register>, // 0x0020_0094
}

///
/// GPPUDCLK peripheral register accessors.
///
#[derive(Default)]
pub struct GPPUD0;

impl ops::Deref for GPPUD0 {
    type Target = RegisterBlockGPPUD;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl GPPUD0 {
    fn ptr() -> *const RegisterBlockGPPUD {
        GPPUD_BASE as *const _
    }

    pub fn pud_uart(&self) {
        self.GPPUD0.write(GPPUD::PUD::OFF);
        for _ in 0..150 { asm::nop(); }
    }
}


/**********************************************************************
 * GPPUDCLK
 *********************************************************************/

register_bitfields! {
    u32,

/// GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [
/// Pin 15 - Set to assert clock.
        PUDCLK15 OFFSET(15) NUMBITS(1) [],

/// Pin 14 - Set to assert clock.
        PUDCLK14 OFFSET(14) NUMBITS(1) []
    ]
}

///
///GPPUDCLK GPIO pin clock enable - 0x7E200098
///
const GPPUDCLK_OFFSET: u32 = 0x0020_0098;
const GPPUDCLK_BASE:   u32 = MMIO_BASE + GPPUDCLK_OFFSET;


///
/// GPPUDCLK peripheral registers
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockGPPUDCLK {
    GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>, // 0x0020_0098
}

///
/// GPPUDCLK peripheral register accessors.
///
#[derive(Default)]
pub struct GPPUDCLK;

impl ops::Deref for GPPUDCLK {
    type Target = RegisterBlockGPPUDCLK;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl GPPUDCLK {
    fn ptr() -> *const RegisterBlockGPPUDCLK {
        GPPUDCLK_BASE as *const _
    }

    pub fn pudclk_uart(&self) {
        self.GPPUDCLK0.modify(GPPUDCLK0::PUDCLK14::SET + 
                              GPPUDCLK0::PUDCLK15::SET);
        for _ in 0..150 { asm::nop(); }
        self.GPPUDCLK0.set(0);
    }
}


/**********************************************************************
 * PL011
 *********************************************************************/

// PL011 UART registers.
//
// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
//
register_bitfields! {
    u32,

/// Data Register
    DR [
        DATA OFFSET(0) NUMBITS(32) []
    ],
    
///Register not documented in datasheet.
    RSRECR [
        UNDOCUMENTED OFFSET(0) NUMBITS(32) []
    ],

/// Flag Register
    FR [
/// Transmit FIFO full. The meaning of this bit depends on the
/// state of the FEN bit in the UARTLCR_ LCRH Register. If the
/// FIFO is disabled, this bit is set when the transmit
/// holding register is full. If the FIFO is enabled, the TXFF
/// bit is set when the transmit FIFO is full.
        TXFF OFFSET(5) NUMBITS(1) [],

/// Receive FIFO empty. The meaning of this bit depends on the
/// state of the FEN bit in the UARTLCR_H Register. If the
/// FIFO is disabled, this bit is set when the receive holding
/// register is empty. If the FIFO is enabled, the RXFE bit is
/// set when the receive FIFO is empty.
        RXFE OFFSET(4) NUMBITS(1) []
    ],

///Register unused by hardware implementation of PL011.
    ILPR [
        UNUSED OFFSET(0) NUMBITS(32) []
    ],

/// Integer Baud rate divisor
    IBRD [
/// Integer Baud rate divisor
        IBRD OFFSET(0) NUMBITS(16) []
    ],

/// Fractional Baud rate divisor
    FBRD [
/// Fractional Baud rate divisor
        FBRD OFFSET(0) NUMBITS(6) []
    ],

/// Line Control register
    LCRH [
/// Word length. These bits indicate the number of data bits
/// transmitted or received in a frame.
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ]
    ],

/// Control Register
    CR [
/// Receive enable. If this bit is set to 1, the receive
/// section of the UART is enabled. Data reception occurs for
/// UART signals. When the UART is disabled in the middle of
/// reception, it completes the current character before
/// stopping.
        RXE    OFFSET(9) NUMBITS(1) [],

/// Transmit enable. If this bit is set to 1, the transmit
/// section of the UART is enabled. Data transmission occurs
/// for UART signals. When the UART is disabled in the middle
/// of transmission, it completes the current character before
/// stopping.
        TXE    OFFSET(8) NUMBITS(1) [],

/// UART enable
/// If the UART is disabled in the middle of transmission
/// or reception, it completes the current character
/// before stopping.
        UARTEN OFFSET(0) NUMBITS(1) []
    ],

/// Interupt FIFO Level Select Register. FIXME: Unimplemented.
    IFLS [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],

/// Interupt Mask Set Clear Register. FIXME: Unimplemented.
    IMSC [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],

/// Raw Interupt Status Register. FIXME: Unimplemented.
    RIS [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],

/// Masked Interupt Status Register. FIXME: Unimplemented.
    MIS [
        UNIMPLEMENTED OFFSET(0) NUMBITS(32) []
    ],
    
/// Interupt Clear Register
    ICR [
/// Meta field for all pending interrupts
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

const UART0_BASE: u32 = MMIO_BASE + 0x0020_1000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockPL011 {
///Data register.
    DR:     ReadWrite<u32, DR::Register>,       // 0x00

///Undocumented
    RSRECR: ReadWrite<u32, RSRECR::Register>,   // 0x04

///Reserved 0
    __res0: [u32; 4],                           // 0x08 - 0x014 not assigned.

///Flag register
    FR:     ReadWrite<u32, FR::Register>,       // 0x18
    
///Reserved 1
    __res1: [u32; 1],                           // 0x1c not assigned.

///Not in use
    ILPR:   ReadWrite<u32, ILPR::Register>,     // 0x20

///Integer Baud rate divisor
    IBRD:   ReadWrite<u32, IBRD::Register>,     // 0x24

///Fractional Baud rate divisor
    FBRD:   ReadWrite<u32, FBRD::Register>,     // 0x28
    
///Line Control register
    LCRH:   ReadWrite<u32, LCRH::Register>,     // 0x2C
    
///Control register
    CR:     ReadWrite<u32, CR::Register>,       // 0x30

///Interupt FIFO Level Select Register
    IFLS:   ReadWrite<u32, ILPR::Register>,     // 0x34

///Interupt Mask Set Clear Register
    IMSC:   ReadWrite<u32, ILPR::Register>,     // 0x38
    
///Raw Interupt Status Register
    RIS:    ReadWrite<u32, ILPR::Register>,     // 0x3C

///Masked Interupt Status Register
    MIS:    ReadWrite<u32, ILPR::Register>,     // 0x40

///Interupt Clear Register
    ICR:    ReadWrite<u32, ICR::Register>,      // 0x44
}


#[derive(Default)]
pub struct Uart0;

impl ops::Deref for Uart0 {
    type Target = RegisterBlockPL011;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl Uart0 {

/// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlockPL011 {
        UART0_BASE as *const _
    }

    pub fn init() {
        let uart = Uart0::default();
        let mut mbox = mbox::Mbox::default();

// turn off UART0
        uart.CR.write(CR::UARTEN::CLEAR);

// set up clock for consistent divisor values
        mbox.buffer[0] = 9 * 4;
        mbox.buffer[1] = mbox::REQUEST;
        mbox.buffer[2] = mbox::tag::SETCLKRATE;
        mbox.buffer[3] = 12;
        mbox.buffer[4] = 8;
        mbox.buffer[5] = mbox::clock::UART; // UART clock
        mbox.buffer[6] = 4_000_000; // 4Mhz
        mbox.buffer[7] = 0; // skip turbo setting
        mbox.buffer[8] = mbox::tag::LAST;

// Insert a compiler fence that ensures that all stores to the
// mbox buffer are finished before the GPU is signaled (which
// is done by a store operation as well).
        compiler_fence(Ordering::Release);

        if let Err(err) = mbox.call(mbox::channel::PROP) {
            match err {
                mbox::MboxError::ResponseError => { panic!(); }
                mbox::MboxError::UnknownError  => { }
            }
        }

//Select Uart0 (PL011) alternate function for GPIO pins. 
        GPFSEL::default().fsel_uart0();
//Set pull up/down control signal in register.
        GPPUD0::default().pud_uart();
//Set the Uart pins that control signal applies to.
        GPPUDCLK::default().pudclk_uart();

//Set up Uart0 for 155200 baud, 8N1 operation.
        uart.ICR.modify(ICR::ALL::CLEAR);
        uart.IBRD.modify(IBRD::IBRD.val(2)); // Results in 115200 baud
        uart.FBRD.modify(FBRD::FBRD.val(0xB));
        uart.LCRH.modify(LCRH::WLEN::EightBit); // 8N1

//Enable UART, RX, and TX
        uart.CR.write(CR::UARTEN::SET + 
                      CR::TXE::SET    + 
                      CR::RXE::SET);
    }

    ///Poll for pending character.
    pub fn poll_tx(&self) -> bool {
        !self.FR.is_set(FR::TXFF)
    }

    ///Poll for pending character.
    pub fn poll_rx(&self) -> bool {
        !self.FR.is_set(FR::RXFE)
    }

    /// Send a character
    pub fn send(&self, c: char) {
        // wait until we can send
        loop {
            if !self.FR.is_set(FR::TXFF) {
                break;
            }

            asm::nop();
        }

        // write the character to the buffer
        self.DR.set(c as u32);
    }

    /// Receive an unsigned 8-bit int.
    pub fn getu8(&self) -> u8 {
        // wait until something is in the buffer
        loop {
            if !self.FR.is_set(FR::RXFE) {
                break;
            }

            asm::nop();
        }

        // read it and return
        self.DR.get() as u8
    }
    
    /// Receive a character
    pub fn getc(&self) -> char {
        // wait until something is in the buffer
        loop {
            if !self.FR.is_set(FR::RXFE) {
                break;
            }

            asm::nop();
        }

        // read it and return
        let mut ret = self.DR.get() as u8 as char;

        // convert carrige return to newline
        if ret == '\r' {
            ret = '\n'
        }

        ret
    }

    /// Display a string
    pub fn puts(&self, string: &str) {
        for c in string.chars() {
            // convert newline to carrige return + newline
            if c == '\n' {
                self.send('\r')
            }

            self.send(c);
        }
    }

    /// Display a binary value in hexadecimal
    pub fn hex32(&self, d: u32) {
        let mut n;

        for i in 0..8 {
            // get highest tetrad
            n = d.wrapping_shr(28 - i * 4) & 0xF;

            // 0-9 => '0'-'9', 10-15 => 'A'-'F'
            // Add proper offset for ASCII table
            if n > 9 {
                n += 0x37;
            } else {
                n += 0x30;
            }

            self.send(n as u8 as char);
        }
    }
}
