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
use core::ops;
use register::register_bitfields;
use register::mmio::ReadWrite;

///
///DMA0 registers control the DMA peripheral.
///
const DMA0_OFFSET:     u32 = 0x0000_07000;
const DMA0_BASE:       u32 = MMIO_BASE + DMA0_OFFSET;
const INT_STATUS_BASE: u32 = DMA0_BASE + 0xFE0;
const ENABLE_BASE:     u32 = DMA0_BASE + 0xFF0;


register_bitfields! {
    u32,

///CS - Control and status. 0-14_CS
    CS [
///Write 1 to reset DMA channel.
        RESET OFFSET(31) NUMBITS(1) [],
///Write 1 to abort current control block and move to the next.
        ABORT OFFSET(30) NUMBITS(1) [],
///Disable debug pause.
        DISDEBUG  OFFSET(29) NUMBITS(1) [],
///Wait until all AXI writes have been acknowledged before loading next CB.
        WAIT_FOR_OUTSTANDING_WRITES OFFSET(28) NUMBITS(1) [],
///Priority of panic AXI bus transactions, lowest is 0x0.
        PANIC_PRIORITY OFFSET(20) NUMBITS(4) [],
///Priority of normal AXI bus transactions, lowest 0x0.
        PRIORITY OFFSET(16) NUMBITS(4) [],
///Indicates if the DMA has detected an error. Details are in the DEBUG register.
        ERROR OFFSET(8) NUMBITS(1) [],
///Set to 1 when DMA is waiting for writes to complete.
        WAITING_FOR_OUTSTANDING_WRITES OFFSET(6) NUMBITS(1) [],
///Reads 1 when DREQ is low indicating no data.
        DREQ_STOPS_DMA OFFSET(5) NUMBITS(1) [],
///Reads 1 if the channel is paused.
        PAUSED OFFSET(4) NUMBITS(1) [],
///Reads 1 if requesting data.
        DREQ OFFSET(3) NUMBITS(1) [],
///Reads 1 if interrupt has occurred. Write 1 to clear.
        INT OFFSET(2) NUMBITS(1) [],
///Set when current CB transfer is done. Write 1 to clear.
        END OFFSET(1) NUMBITS(1) [],
///Set SCB_ADDR and write 1 to begin DMA transfer. Clear to pause.
        ACTIVE OFFSET(0) NUMBITS(1) []
    ],

///DMA Control Block Address register 0-14_CONBLK_AD
    CONBLK_AD [
        SCB_ADDR OFFSET(0) NUMBITS(32) []
    ],

///TI - Transfer information 0-6_TI
    TI [
///Turn off wide writes with 2 beat bursts. (?)
        NO_WIDE_BURSTS OFFSET(26) NUMBITS(1) [],
///Add wait cycles.
        WAITS OFFSET(21) NUMBITS(5) [
            NONE = 0b00000
        ],
///Indicates the peripheral to listen for DREQ and Panic signals.
        PERMAP OFFSET(16) NUMBITS(5) [
            UNPACED          = 0,
            DSI0             = 1,
            PCM_TX           = 2,
            PCM_RX           = 3,
            SMI              = 4,
            PWM              = 5,
            SPI_TX           = 6,
            SPI_RX           = 7,
            BSC_SPI_SLAVE_TX = 8,
            BSC_SPI_SLAVE_RX = 9,
            UNUSED           = 10,
            EMMC             = 11,
            UART_TX          = 12,
            SD_HOST          = 13,
            UART_RX          = 14,
            DSI1             = 15,
            SLIMBUS_MCTX     = 16,
            HDMI             = 17,
            SLIMBUS_MCRX     = 18,
            SLIMBUS_DC0      = 19,
            SLIMBUS_DC1      = 20,
            SLIMBUS_DC2      = 21,
            SLIMBUS_DC3      = 22,
            SLIMBUS_DC4      = 23,
            SCALER_FIFO_0    = 24,
            SCALER_FIFO_1    = 25,
            SCALER_FIFO_2    = 26,
            SLIMBUS_DC5      = 27,
            SLIMBUS_DC6      = 28,
            SLIMBUS_DC7      = 29,
            SLIMBUS_DC8      = 30,
            SLIMBUS_DC9      = 31
        ],
///Attempt to transfer this number of words per burst. Zero sets to a single transfer.
        BURST_LENGTH OFFSET(12) NUMBITS(4) [
            SINGLE = 0b0000
        ],
///If set does not perform src reads.
        SRC_IGNORE OFFSET(11) NUMBITS(1) [],
///Enable DREQ control set by PERMAP.
        SRC_DREQ OFFSET(10) NUMBITS(1) [],
///Source transfer read width.
        SRC_WIDTH OFFSET(9) NUMBITS(1) [
            B32  = 0b0, //Use 32 bit reads.
            B128 = 0b1  //Use 128 bit reads.
        ],
///Source address increments after each read by SRC_WIDTH amt.
        SRC_INC OFFSET(8) NUMBITS(1) [],
///Ignore writes. If set does not perform writes to destination.
        DEST_IGNORE OFFSET(7) NUMBITS(1) [],
///Enable DREQ control set by PERMAP.
        DEST_DREQ OFFSET(6) NUMBITS(1) [],
///Destination transfer write width.
        DEST_WIDTH OFFSET(5) NUMBITS(1) [
            B32  = 0b0, //Use 32 bit reads.
            B128 = 0b1  //Use 128 bit reads.
        ],
///Destination address increments after each write
        DEST_INC OFFSET(4) NUMBITS(1) [],
///Wait for AXI response for each write instead of pipelining.
        WAIT_RESP OFFSET(3) NUMBITS(1) [],
///Two Dimensional mode. Interpret TXFR_LEN as [x][y] array.
        TDMODE OFFSET(1) NUMBITS(1) [],
///Interrupt enable - generate an interrupt when current block transfer completes.
        INTEN OFFSET(0) NUMBITS(1) []
    ],

///SOURCE_AD - Source address 0-14_SOURCE_AD
    SOURCE_AD [
        ADDR OFFSET(0) NUMBITS(32) []
    ],

///DEST_AD - Destination address 0-14_DEST_AD
    DEST_AD [
        ADDR OFFSET(0) NUMBITS(32) []
    ],

///TXFR_LEN - Transfer length. 0-6_TXFR_LEN
    TXFR_LEN [
///If 2D array [x][y] number of rows.
        YLENGTH OFFSET(16) NUMBITS(14) [],
///If 1D array [x] or 2D array [x][y] length in bytes.
        XLENGTH OFFSET(0) NUMBITS(16) []
    ],

///STRIDE - 2D mode stride. 0-6_STRIDE
    STRIDE [
///Signed (2 s complement) byte increment to apply to the destination 
///address at the end of each row in 2D mode.
        D_STRIDE OFFSET(16) NUMBITS(16) [],
///Signed (2 s complement) byte increment to apply to the source
///address at the end of each row in 2D mode.
        S_STRIDE OFFSET(0) NUMBITS(16) []
    ],

///NEXTCONBK - Next control block address. 0-14_NEXTCONBK
    NEXTCONBK [
        ADDR OFFSET(0) NUMBITS(32) []
    ],

///DEBUG - Debugging information. 0-6_DEBUG
    DEBUG [
///Set if this channel is a LITE DMA channel.
         LITE OFFSET(28) NUMBITS(1) [],
///DMA version number
         VERSION OFFSET(25) NUMBITS(3) [],
///DMA machine state.
         DMA_STATE OFFSET(16) NUMBITS(9) [],
///AXI ID of this channel.
         DMA_ID OFFSET(8) NUMBITS(8) [],
///Number of write responses that have not been received.
         OUTSTANDING_WRITES OFFSET(4) NUMBITS(4) [],
///Set if there was an error on read.
         READ_ERROR OFFSET(2) NUMBITS(1) [],
///Set if optional read FIFO had an error.
         FIFO_ERROR OFFSET(1) NUMBITS(1) [],
///AXI read last was not set when expected.
         READ_LAST_NOT_SET_ERROR OFFSET(0) NUMBITS(1) []
    ],
    
    INT_STATUS [
        INT15 OFFSET(15) NUMBITS(1) [],
        INT14 OFFSET(14) NUMBITS(1) [],
        INT13 OFFSET(13) NUMBITS(1) [],
        INT12 OFFSET(12) NUMBITS(1) [],
        INT11 OFFSET(11) NUMBITS(1) [],
        INT10 OFFSET(10) NUMBITS(1) [],
        INT9 OFFSET(9) NUMBITS(1) [],
        INT8 OFFSET(8) NUMBITS(1) [],
        INT7 OFFSET(7) NUMBITS(1) [],
        INT6 OFFSET(6) NUMBITS(1) [],
        INT5 OFFSET(5) NUMBITS(1) [],
        INT4 OFFSET(4) NUMBITS(1) [],
        INT3 OFFSET(3) NUMBITS(1) [],
        INT2 OFFSET(2) NUMBITS(1) [],
        INT1 OFFSET(1) NUMBITS(1) [],
        INT0 OFFSET(0) NUMBITS(1) []
    ],
    
    ENABLE [
        EN14 OFFSET(14) NUMBITS(1) [],
        EN13 OFFSET(13) NUMBITS(1) [],
        EN12 OFFSET(12) NUMBITS(1) [],
        EN11 OFFSET(11) NUMBITS(1) [],
        EN10 OFFSET(10) NUMBITS(1) [],
        EN9 OFFSET(9) NUMBITS(1) [],
        EN8 OFFSET(8) NUMBITS(1) [],
        EN7 OFFSET(7) NUMBITS(1) [],
        EN6 OFFSET(6) NUMBITS(1) [],
        EN5 OFFSET(5) NUMBITS(1) [],
        EN4 OFFSET(4) NUMBITS(1) [],
        EN3 OFFSET(3) NUMBITS(1) [],
        EN2 OFFSET(2) NUMBITS(1) [],
        EN1 OFFSET(1) NUMBITS(1) [],
        EN0 OFFSET(0) NUMBITS(1) [],
        RAW OFFSET(0) NUMBITS(32) []
    ]
}

/**********************************************************************
 * ControlBlockDMA
 *********************************************************************/

#[repr(C)]
#[repr(align(32))] //Blocks must be aligned to a 256 bit (32 byte) boundary.
#[allow(non_snake_case)]
#[derive(Default, Copy, Clone)]
pub struct ControlBlock {
    pub TI:        u32,
    pub SOURCE_AD: u32,
    pub DEST_AD:   u32,
    pub TXFR_LEN:  u32,
    pub STRIDE:    u32,
    pub NEXTCONBK: u32,
    __RES0:        u32,
    __RES1:        u32,
}

/**********************************************************************
 * RegisterBlockDMA
 *********************************************************************/

#[repr(C)]
#[allow(non_snake_case)]
pub struct RegisterBlockDMA {
    pub CS:        ReadWrite<u32, CS::Register>,
    pub CONBLK_AD: ReadWrite<u32, CONBLK_AD::Register>,
    pub TI:        ReadWrite<u32, TI::Register>,
    pub SOURCE_AD: ReadWrite<u32, SOURCE_AD::Register>,
    pub DEST_AD:   ReadWrite<u32, DEST_AD::Register>,
    pub TXFR_LEN:  ReadWrite<u32, TXFR_LEN::Register>,
    pub STRIDE:    ReadWrite<u32, STRIDE::Register>,
    pub NEXTCONBK: ReadWrite<u32, NEXTCONBK::Register>,
    pub DEBUG:     ReadWrite<u32, DEBUG::Register>,
    __RES0:        [u8; 220], //9 * 4 + 220 = 256 
}

/**********************************************************************
 * INT_STATUS_INTERNAL
 *********************************************************************/

///
///Points at the DMA interrupt status register.
///
#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct INT_STATUS_INTERNAL;

impl ops::Deref for INT_STATUS_INTERNAL {
    type Target = ReadWrite<u32, INT_STATUS::Register>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl INT_STATUS_INTERNAL {
    #[inline]
    fn ptr() -> *const ReadWrite<u32, INT_STATUS::Register> {
        INT_STATUS_BASE as *const _
    }
}


/**********************************************************************
 * ENABLE_INTERNAL
 *********************************************************************/

///
///Points at the DMA enable register.
///
#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct ENABLE_INTERNAL;

impl ops::Deref for ENABLE_INTERNAL {
    type Target = ReadWrite<u32, ENABLE::Register>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl ENABLE_INTERNAL {
    #[inline]
    fn ptr() -> *const ReadWrite<u32, ENABLE::Register> {
        ENABLE_BASE as *const _
    }
}


/**********************************************************************
 * RegisterMapDMA
 *********************************************************************/

///
///Puts DMA channels 0-14, interrupt status and enable registers in one
///structure.
///
#[repr(C)]
#[allow(non_snake_case)]
pub struct RegisterMapDMA {
    pub CHANNELS:  [RegisterBlockDMA; 15],
    pub INT_STATUS: INT_STATUS_INTERNAL,
    pub ENABLE: ENABLE_INTERNAL,
}


/**********************************************************************
 * DMA
 *********************************************************************/

#[derive(Default)]
pub struct DMA;

impl ops::Deref for DMA {
    type Target = RegisterMapDMA;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}


impl DMA {
    #[inline]
    fn ptr() -> *const RegisterMapDMA {
        DMA0_BASE as *const _
    }
}
