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
 * RPi3 I2C GPIO Pin Setup
 *
 * Reference:
 *  https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
 * 
 * I2C0 - Don't use I2C0 on RPi3!
 *  GPIO Pin 0 - Pull High, Alternative function 0 = SDA0
 *  GPIO Pin 1 - Pull High, Alternative function 0 = SCL0
 *
 * I2C1
 *  GPIO Pin 2 - Pull High, Alternative function 0 = SDA1
 *  GPIO Pin 3 - Pull High, Alternative function 0 = SCL1
 * 
 * Select I2C1 For GPIO Pin Function Pins 2 & 3:
 *
 * GPFSEL0 alternative function select register - 0x7E200000
 *  GPIO Pin 2 alternative function select - 3 bits [8..6]  = 0b100
 *  GPIO Pin 3 alternative function select - 3 bits [11..9] = 0b100
 *
 *
 * Control I2C1 from BSC1.
 *  BSC1 serial controller register - 0x7E804000
 *  C control                0x0
 *  S Status                 0x4
 *  DLEN Data length         0x8
 *  A Slave address          0xC
 *  FIFO Data fifo           0x10
 *  DIV Clk divider          0x14
 *  DEL Data delay           0x18
 *  CLKT Clk stretch timeout 0x1C
 *
 */

use super::MMIO_BASE;
use register::register_bitfields;
use register::mmio::ReadWrite;
use core::ops;
use crate::debug;


/**********************************************************************
 * ERROR
 *********************************************************************/

pub enum ERROR {
///BSC is active.
    ACTIVE,
///BSC is not active
    INACTIVE,
///Slave did not acknowledge a transfer.
    NOACK,      
///Slave did not reply in time.
    TIMEOUT,
///I2C transaction is unexpectedly done.
    DONE
}

impl ERROR {
    pub fn msg (&self) -> &'static str {
        match self {
            ERROR::ACTIVE   => "BSC is unexpectedly active.",
            ERROR::INACTIVE => "BSC is unexpectedly inactive.",
            ERROR::NOACK    => "Slave did not acknowledge a transfer.",
            ERROR::TIMEOUT  => "Slave did not reply in time.",
            ERROR::DONE     => "Transaction is unexpectedly done."
        }
    }
}


/**********************************************************************
 * GPFSEL
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
struct RegisterBlockGPFSEL {
    GPFSEL0: ReadWrite<u32, GPFSEL0::Register>
}

///
///Implements accessors to the GPFSEL registers. 
///
struct GPFSEL;

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

    const fn new() -> GPFSEL {
        GPFSEL
    }

///
///Select alternate GPIO pin functions for the I2C1 peripheral.
///
    fn fsel_i2c1(&self) {
        self.GPFSEL0.modify(GPFSEL0::FSEL2::INPUT + 
                            GPFSEL0::FSEL3::INPUT);

        self.GPFSEL0.modify(GPFSEL0::FSEL2::SDA1 + 
                            GPFSEL0::FSEL3::SCL1);
    }
}


/**********************************************************************
 * BSC
 *********************************************************************/

register_bitfields! {
    u32,

///C control offset 0x0
    C [
///Enable I2C
        I2CEN OFFSET(15) NUMBITS(1) [],

///Generate interrupt on receive.
        INTR OFFSET(10) NUMBITS(1) [],

///Generate interrupt on transmit.
        INTT OFFSET(9) NUMBITS(1) [],

///Generate interrupt on done.
        INTD OFFSET(8) NUMBITS(1) [],

///Start transfer.
        START OFFSET(7) NUMBITS(1) [],

///Clear FIFO of data.
        CLEAR OFFSET(4) NUMBITS(2) [], 

///Read/Write packet transfer.
        READ OFFSET(0) NUMBITS(1) []
    ],

 ///S Status 0x4
    S [
///Clock stretch timeout. There may be hardware bugs which affect/limit clock stretch.
        CLKT OFFSET(9) NUMBITS(1) [],
        
///Slave has not acknowledged address
        ERR OFFSET(8) NUMBITS(1) [],
        
///FIFO is full and can not receive more data.
        RXF OFFSET(7) NUMBITS(1) [],

///FIFO is empty and no data can be transmitted.
        TXE OFFSET(6) NUMBITS(1) [],
        
///FIFO contains data that can be read.
        RXD OFFSET(5) NUMBITS(1) [],
        
///FIFO is not full and can be written to.
        TXD OFFSET(4) NUMBITS(1) [],
        
///FIFO is full and should be read.
        RXR OFFSET(3) NUMBITS(1) [],
        
///FIFO is not full and should be written to.
        TXW OFFSET(2) NUMBITS(1) [],
        
///Transfer done.
        DONE OFFSET(1) NUMBITS(1) [],
        
///Transfer active
        TA OFFSET(0) NUMBITS(1) []
    ],

///DLEN Data length 0x8
    DLEN [
        DLEN OFFSET(0) NUMBITS(16) []
    ],

///A Slave address 0xC
    A [
        ADDR OFFSET(0) NUMBITS(7) []
    ],

///FIFO Data fifo 0x10
    FIFO [
        DATA OFFSET(0) NUMBITS(8) []
    ],

///DIV Clk divider 0x14
    DIV [
        CDIV OFFSET(0) NUMBITS(16) []
    ],

///Edge delay in clocks 0x18.
    DEL [
///Falling edge delay in clocks.
        FEDL OFFSET(16) NUMBITS(16) [],

///Rising edge delay in clocks.
        REDL OFFSET(0) NUMBITS(16) []
    ],

///Clock stretch timeout 0x1C
    CLKT [
        TOUT OFFSET(0) NUMBITS(16) []
    ]
}

///
///BSC1 registers control the I2C1 peripheral.
///
const BSC1_OFFSET:  u32 = 0x0080_4000;
const BSC1_BASE:    u32 = MMIO_BASE + BSC1_OFFSET; 


///
///Register block representing all the BSC registers.
///
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockBSC {
///Control register
    C:      ReadWrite<u32, C::Register>,
    
///Status register.
    S:      ReadWrite<u32, S::Register>,

///Number of bytes of data to transmit or receive.
    DLEN:   ReadWrite<u32, DLEN::Register>,

///Address of slave peripheral on I2C bus.
    A:      ReadWrite<u32, A::Register>,

///FIFO read/write access.
    FIFO:   ReadWrite<u32, FIFO::Register>,

///Clock divider sets the I2C bus speed.
    DIV:    ReadWrite<u32, DIV::Register>,

///Allows for fine tuning the rise and falling times on the I2C bus.
    DEL:    ReadWrite<u32, DEL::Register>,

///Timeout in clock cycles before current transfer fails.
    CLKT:   ReadWrite<u32, CLKT::Register>
}


/**********************************************************************
 * I2C
 *********************************************************************/

pub trait I2C {
    fn ptr() -> *const RegisterBlockBSC;
    fn init();
    fn init_internal(&self);
    fn reset(&self);
    fn poll_error(&self) -> Result<(), ERROR>;
    fn poll_done(&self) -> Result<(), ERROR>;
    fn write(&self, addr: u8, reg: u8, data: &[u8]) -> Result<(), ERROR>;
    fn read(&self, addr: u8, reg: u8, data: &mut [u8]) -> Result<(), ERROR>;
}

/**********************************************************************
 * I2C1
 *********************************************************************/

///
/// Second I2C (I2C1) peripheral registers
///
#[derive(Default)]
pub struct I2C1;

impl ops::Deref for I2C1 {
    type Target = RegisterBlockBSC;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl I2C for I2C1 {
    fn ptr() -> *const RegisterBlockBSC {
        BSC1_BASE as *const _
    }

    fn init() { 
        I2C1::default().init_internal();
    }

    fn init_internal(&self) {
        GPFSEL::new().fsel_i2c1(); //Select the GPIO pins for I2C1.

        self.DIV.modify(DIV::CDIV.val(0xFFFF));   //Value of 0 defaults to divsor of 32768.
        self.CLKT.modify(CLKT::TOUT.val(0));      //Turn off clock stretching since it's buggy.

        self.C.write (
            C::I2CEN::SET + //Enable I2C
            C::CLEAR::SET   //Clear the FIFO.
        );

        self.S.modify (
            S::CLKT::SET + //Reset clock timeout.
            S::ERR::SET  + //Reset err.
            S::DONE::SET   //Reset done.
        );
    }

///Reset i2c regardless of transfer active status.
    fn reset(&self) {
//Reset status register.
        self.S.modify (
            S::CLKT::SET +
            S::ERR::SET  +
            S::DONE::SET
        );
//Clear the FIFO.
        self.C.modify(C::CLEAR::SET);
    }

///Determine if an active transfer has an error condition.
    fn poll_error(&self) -> Result<(), ERROR> {
        let s = self.S.extract();
//Poll for error.
        if s.is_set(S::ERR) {
            return Err(ERROR::NOACK);
        }
//Poll for timeout.
        if s.is_set(S::CLKT) {
            return Err(ERROR::TIMEOUT);
        }
        return Ok(());
    }

    fn poll_done(&self) -> Result<(), ERROR> {
        loop {
            let s = self.S.extract();
//Poll for error.
            if s.is_set(S::ERR) {
                return Err(ERROR::NOACK);
            }
//Poll for timeout.
            if s.is_set(S::CLKT) {
                return Err(ERROR::TIMEOUT);
            }
//Poll for done.
            if s.is_set(S::DONE) {
                break;
            }
        }
        return Ok(());
    }

    fn write(&self, addr: u8, reg: u8, data: &[u8]) -> Result<(), ERROR> {
        let len: usize = data.len();
        let mut i: usize = 0;

//First byte of the FIFO is the slave device register address.
        if self.S.is_set(S::TA) {        //I2C is already in a transfer.
            return Err(ERROR::ACTIVE);
        }
    
//Initialize i2c1
        debug::out("i2c.write(): Init.\r\n");
        self.DLEN.set((len + 1) as u32); //Set data length including register address.
        self.A.set(addr as u32);         //Set the slave address.
        self.C.modify(C::READ::CLEAR);   //Clear READ bit.

        if let Err(err) = self.poll_error() {
            debug::out("i2c.write(): poll_error().\r\n");
            return Err(err);
        }

//Write slave register address.
        debug::out("i2c.write(): Write regaddr.\r\n");
        self.FIFO.set(reg as u32);

//Start transfer.
        debug::out("i2c.write(): Start xfer.\r\n");
        self.C.modify(C::START::SET);
        debug::out("i2c.write(): Xfer started.\r\n");

//Keep the FIFO filled until error or all bytes written.
        while i < len {
            if let Err(err) = self.poll_error() { //Error condition.
                debug::out("i2c.write(): Error.\r\n");
                return Err(err);
            }

            while self.S.matches_all(S::TXD::SET + 
                                        S::TXW::SET) 
            {  //FIFO not full and needs writing.
                assert!(i < len);
                debug::out(".");
                self.FIFO.set(data[i] as u32);
                i += 1;
            }
        }

//Wait for all bytes to be sent to slave.
        debug::out("i2c.write(): Xfer finished. Poll until done.\r\n");
        return self.poll_done();
    }

    fn read(&self, addr: u8, reg: u8, data: &mut [u8]) -> Result<(), ERROR> {
        let len: usize = data.len();
        let mut i: usize = 0;

        if self.S.is_set(S::TA) {        //I2C is already in a transfer.
            return Err(ERROR::ACTIVE);
        }

//Write the slave device register address to read from.
        debug::out("i2c.read(): Write register.\r\n");
        if let Err(err) = self.write(addr, reg, &[]) {
            return Err(err);
        }
        debug::out("i2c.read(): Register written.\r\n");


//Initialize and read.
        debug::out("i2c.read(): Reset and initialize.\r\n");
        self.reset();
        self.DLEN.set(len as u32);       //Set data length including register address.
        self.A.set(addr as u32);         //Set the slave address.
        self.C.modify(C::READ::SET);     //Set BSC to read operation.

        debug::out("i2c.read(): Start xfer.\r\n");
        self.C.modify(C::CLEAR::SET +    //Clear FIFO.
                      C::START::SET);    //Start transfer.
        debug::out("i2c.read(): Xfer started.\r\n");

//Keep the FIFO from overflowing until error or all bytes read.
        while i < len {
            if let Err(err) = self.poll_error() { //Error condition.
                return Err(err);
            }

            while self.S.is_set(S::RXD) { //Read until empty.
                data[i] = self.FIFO.get() as u8;
                i += 1;
            }
        }
        debug::out("i2c.read(): Xfer finished. Poll until done.\r\n");
        return self.poll_done();
    }
}
