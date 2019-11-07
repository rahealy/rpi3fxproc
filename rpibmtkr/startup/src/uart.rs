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

/*
 * Minimal implementation of UART for use in debugging startup.
 */

use super::MMIO_BASE;
use crate::mbox;
use core::ops;
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::asm;
use register::{mmio::*, register_bitfields};


register_bitfields! {
    u32,

///Place holder for struct alignment.
    GPFSEL0 [
        RESERVED OFFSET(0) NUMBITS(32)
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
        ]
    ]
}


///
///GPFSEL0 alternative function select register - 0x7E200000
///
const GPFSEL0_OFFSET: u32 = 0x0020_0000;
const GPFSEL0_BASE:   u32 = MMIO_BASE as u32  + GPFSEL0_OFFSET;


///
///Register block representing all the GPFSEL registers.
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockGPFSEL {
    pub GPFSEL0: ReadWrite<u32, GPFSEL0::Register>, // 0x00200000
    pub GPFSEL1: ReadWrite<u32, GPFSEL1::Register>, // 0x00200004
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

    pub fn fsel_uart0(&self) {
        self.GPFSEL1.modify(GPFSEL1::FSEL14::INPUT + 
                            GPFSEL1::FSEL15::INPUT);
        self.GPFSEL1.modify(GPFSEL1::FSEL14::TXD0 + 
                            GPFSEL1::FSEL15::RXD0);
    }

//     pub fn fsel_uart1(&self) {
//         self.GPFSEL1.modify(GPFSEL1::FSEL14::INPUT + 
//                             GPFSEL1::FSEL15::INPUT);
//         self.GPFSEL1.modify(GPFSEL1::FSEL14::TXD1 + 
//                             GPFSEL1::FSEL15::RXD1);
//     }
}


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
const GPPUD_BASE:   u32 = MMIO_BASE as u32  + GPPUD_OFFSET;


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
const GPPUDCLK_BASE:   u32 = MMIO_BASE as u32  + GPPUDCLK_OFFSET;


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

const UART0_BASE: u32 = MMIO_BASE as u32  + 0x0020_1000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockPL011 {
///Data register.
    DR:     ReadWrite<u32, DR::Register>,       // 0x00

///Undocumented
    RSRECR: ReadOnly<u32, RSRECR::Register>,    // 0x04

///Reserved 0
    __res0: [u32; 4],                           // 0x08 - 0x014 not assigned.

///Flag register
    FR:     ReadOnly<u32, FR::Register>,        // 0x18
    
///Reserved 1
    __res1: [u32; 1],                           // 0x1c not assigned.

///Not in use
    ILPR:   ReadOnly<u32, ILPR::Register>,      // 0x20

///Integer Baud rate divisor
    IBRD:   WriteOnly<u32, IBRD::Register>,     // 0x24

///Fractional Baud rate divisor
    FBRD:   WriteOnly<u32, FBRD::Register>,     // 0x28
    
///Line Control register
    LCRH:   WriteOnly<u32, LCRH::Register>,     // 0x2C
    
///Control register
    CR:     WriteOnly<u32, CR::Register>,       // 0x30

///Interupt FIFO Level Select Register
    IFLS:   ReadOnly<u32, ILPR::Register>,      // 0x34

///Interupt Mask Set Clear Register
    IMSC:   ReadOnly<u32, ILPR::Register>,      // 0x38
    
///Raw Interupt Status Register
    RIS:    ReadOnly<u32, ILPR::Register>,      // 0x3C

///Masked Interupt Status Register
    MIS:    ReadOnly<u32, ILPR::Register>,      // 0x40

///Interupt Clear Register
    ICR:    WriteOnly<u32, ICR::Register>,      // 0x44
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
        uart.CR.set(0);

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

        if mbox.call(mbox::channel::PROP).is_err() {
            panic!();
        };

//Select Uart0 (PL011) alternate function for GPIO pins. 
        GPFSEL::default().fsel_uart0();
//Set pull up/down control signal in register.
        GPPUD0::default().pud_uart();
//Set the Uart pins that control signal applies to.
        GPPUDCLK::default().pudclk_uart();

//Set up Uart0 for 155200 baud, 8N1 operation.
        uart.ICR.write(ICR::ALL::CLEAR);
        uart.IBRD.write(IBRD::IBRD.val(2)); // Results in 115200 baud
        uart.FBRD.write(FBRD::FBRD.val(0xB));
        uart.LCRH.write(LCRH::WLEN::EightBit); // 8N1

//Enable UART, RX, and TX
        uart.CR.write(CR::UARTEN::SET + 
                      CR::TXE::SET + 
                      CR::RXE::SET);

        uart.puts("Uart0::init(): Uart0 initialized.\r\n");
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

    pub fn tohex(val: u8) -> char {
        match val & 0b0000_1111 {
            0x0 => '0',
            0x1 => '1',
            0x2 => '2',
            0x3 => '3',
            0x4 => '4',
            0x5 => '5',
            0x6 => '6',
            0x7 => '7',
            0x8 => '8',
            0x9 => '9',
            0xA => 'A',
            0xB => 'B',
            0xC => 'C',
            0xD => 'D',
            0xE => 'E',
            0xF => 'F',
            _    => ' '
        }
    }

    pub fn u64hex(&self, val: u64) {
        self.puts("0x");
        for i in (0..16).rev() {
            self.send (
                Uart0::tohex((val >> i * 4) as u8)
            );
        }
    }
}
