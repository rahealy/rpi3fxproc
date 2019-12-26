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

register_bitfields! {
    u32,

///CS - Control and status. 0-14_CS
    CS [
///
        RESET OFFSET(31) NUMBITS(1) [],
///
        ABORT OFFSET(30) NUMBITS(1) [],
///
        DISDEBUG  OFFSET(29) NUMBITS(1) [],
///
        WAIT_FOR_OUTSTANDING_WRITES OFFSET(28) NUMBITS(1) [],
///
        PANIC_PRIORITY OFFSET(20) NUMBITS(4) [],
///
        PRIORITY OFFSET(16) NUMBITS(4) [],
///
        ERROR OFFSET(8) NUMBITS(1) [],
///
        WAITING_FOR_OUTSTANDING_WRITES OFFSET(6) NUMBITS(1) [],
///
        DREQ_STOPS_DMA OFFSET(5) NUMBITS(1) [],
///
        PAUSED OFFSET(4) NUMBITS(1) [],
///
        DREQ OFFSET(3) NUMBITS(1) [],
///
        INT OFFSET(2) NUMBITS(1) [],
///
        END OFFSET(1) NUMBITS(1) [],
///
        ACTIVE OFFSET(0) NUMBITS(1) []
    ],

///DMA Control Block Address register 0-14_CONBLK_AD
    CONBLK_AD [],

///TI - Transfer information 0-6_TI
    TI_0_THRU_6 [
///
        NO_WIDE_BURSTS OFFSET(26) NUMBITS(1) [],
///
        WAITS OFFSET(21) NUMBITS(5) [],
///
        PERMAP OFFSET(16) NUMBITS(5) [],
///
        BURST_LENGTH OFFSET(12) NUMBITS(4) [],
///
        SRC_IGNORE OFFSET(11) NUMBITS(1) [],
///
        SRC_DREQ OFFSET(10) NUMBITS(1) [],
///
        SRC_WIDTH OFFSET(9) NUMBITS(1) [],
///
        SRC_INC OFFSET(8) NUMBITS(1) [],
///
        DEST_IGNORE OFFSET(7) NUMBITS(1) [],
///
        DEST_DREQ OFFSET(6) NUMBITS(1) [],
///
        DEST_WIDTH OFFSET(5) NUMBITS(1) [],
///
        DEST_INC OFFSET(4) NUMBITS(1) [],
///
        WAIT_RESP OFFSET(3) NUMBITS(1) [],
///
        TDMODE OFFSET(1) NUMBITS(1) [],
///
        INTEN OFFSET(0) NUMBITS(1) []
    ],

///TI - Transfer information 7-14_TI
    TI_7_THRU_14 [
///
        WAITS OFFSET(21) NUMBITS(5) [],
///
        PERMAP OFFSET(16) NUMBITS(5) [],
///
        BURST_LENGTH OFFSET(12) NUMBITS(4) [],
///
        SRC_IGNORE OFFSET(11) NUMBITS(1) [],
///
        SRC_DREQ OFFSET(10) NUMBITS(1) [],
///
        SRC_WIDTH OFFSET(9) NUMBITS(1) [],
///
        SRC_INC OFFSET(8) NUMBITS(1) [],
///
        DEST_IGNORE OFFSET(7) NUMBITS(1) [],
///
        DEST_DREQ OFFSET(6) NUMBITS(1) [],
///
        DEST_WIDTH OFFSET(5) NUMBITS(1) [],
///
        DEST_INC OFFSET(4) NUMBITS(1) [],
///
        WAIT_RESP OFFSET(3) NUMBITS(1) [],
///
        INTEN OFFSET(0) NUMBITS(1) []
    ],

///SOURCE_AD - Source address 0-14_SOURCE_AD
    SOURCE_AD [],

///DEST_AD - Destination address 0-14_DEST_AD
    DEST_AD [],

///TXFR_LEN_0_THRU_6 - Transfer length. 0-6_TXFR_LEN
    TXFR_LEN_0_THRU_6 [
///
        YLENGTH OFFSET(16) NUMBITS(14) [],
///
        XLENGTH OFFSET(0) NUMBITS(16) []
    ],

///TXFR_LEN_7_THRU_14 - Transfer length. 7-14_TXFR_LEN
    TXFR_LEN_7_THRU_14 [
///
        XLENGTH OFFSET(0) NUMBITS(16) []
    ],

///STRIDE - 2D mode stride. 0-6_STRIDE
    STRIDE_0_THRU_6 [
///
        D_STRIDE OFFSET(16) NUMBITS(16) [],
///
        S_STRIDE OFFSET(0) NUMBITS(16) []
    ],

///NEXTCONBK - Next control block address. 0-14_NEXTCONBK
    NEXTCONBK [],

///DEBUG_0_THRU_6 - Debugging information. 0-6_DEBUG
    DEBUG_0_THRU_6 [
///
         LITE OFFSET(28) NUMBITS(1) [],
///
         VERSION OFFSET(25) NUMBITS(3) [],
///
         DMA_STATE OFFSET(16) NUMBITS(9) [],
///
         DMA_ID OFFSET(8) NUMBITS(8) [],
///
         OUTSTANDING_WRITES OFFSET(4) NUMBITS(4) [],
///
         READ_ERROR OFFSET(2) NUMBITS(1) [],
///
         FIFO_ERROR OFFSET(1) NUMBITS(1) [],
///
         READ_LAST_NOT_SET_ERROR OFFSET(0) NUMBITS(1) []
    ]

///DEBUG_7_THRU_14 - Debugging information. 7-14_DEBUG
    DEBUG_7_THRU_14 [
///
         LITE OFFSET(28) NUMBITS(1) [],
///
         VERSION OFFSET(25) NUMBITS(3) [],
///
         DMA_STATE OFFSET(16) NUMBITS(9) [],
///
         DMA_ID OFFSET(8) NUMBITS(8) [],
///
         OUTSTANDING_WRITES OFFSET(4) NUMBITS(4) [],
///
         READ_ERROR OFFSET(2) NUMBITS(1) [],
///
         FIFO_ERROR OFFSET(1) NUMBITS(1) [],
///
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
        EN0 OFFSET(0) NUMBITS(1) []
    ]
}

///
///A register in the DMA controler is set to a pointer to an 
///initialized in memory ControlBlockDMA structure. Information
///in the block is used to configure the DMA channel.
///
#[allow(non_snake_case)]
#[repr(C)]
#[repr(align(32))]
pub struct ControlBlockDMA {
    TI:        ReadWrite<u32, TI::Register>,
    SOURCE_AD: ReadWrite<u32, SOURCE_AD::Register>,
    DEST_AD:   ReadWrite<u32, DEST_AD::Register>,
    TXFR_LEN:  ReadWrite<u32, TXFR_LEN::Register>,
    STRIDE:    ReadWrite<u32, STRIDE::Register>,
    NEXTCONBK: ReadWrite<u32, NEXTCONBK::Register>,
    __RES0:    u32,
    __RES1:    u32,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlockDMA {
    CS:        ReadWrite<u32, O_CS::Register>,
    CONBLK_AD: ReadWrite<u32, O_CS::Register>,
    TI:        ReadWrite<u32, TI::Register>,
    SOURCE_AD: ReadWrite<u32, SOURCE_AD::Register>,
    DEST_AD:   ReadWrite<u32, DEST_AD::Register>,
    TXFR_LEN:  ReadWrite<u32, TXFR_LEN::Register>,
    STRIDE:    ReadWrite<u32, STRIDE::Register>,
    NEXTCONBK: ReadWrite<u32, NEXTCONBK::Register>,
    DEBUG:     ReadWrite<u32, DEBUG::Register>,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterMapDMA {
    CHANNEL_0_THRU_6:  [RegisterBlockDMA; 7],
    CHANNEL_7_THRU_14: [RegisterBlockDMA; 7],
    INT_STATUS:        ReadWrite<u32, INT_STATUS::Register>,
    ENABLE:            ReadWrite<u32, INT_STATUS::Register>
}
    
#[derive(Default)]
pub struct DMA;

impl ops::Deref for DMA {
    type Target = RegisterMapDMA;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

///
///DMA0 registers control the DMA peripheral.
///
const DMA0_OFFSET:  u32 = 0x0000_07000;
const DMA0_BASE:    u32 = MMIO_BASE + DMA0_OFFSET;

impl DMA {
    #[inline]
    fn ptr() -> *const RegisterBlockDMA {
        DMA0_BASE as *const _
    }
}
