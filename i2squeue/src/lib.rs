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
#![no_std]

///
///Helper module provides initialization and a rx/tx queue for samples
///read from and ready to send to the I2S bus.
///

//use register::mmio::ReadWrite;
use peripherals::i2s;
//use peripherals::i2s::I2S;
use peripherals::dma::*;
use peripherals::debug;
//use rack::effect::SampleType;
//use common::buffer::{Queue, Read, Write, Amount};
//use common::buffer::{Queue};

const I2S_FIFO: u32 = i2s::PCM_BASE + 0x4; //FIFO Buffer at offset 0x4.
const BUFFER_LEN: usize = i2s::FIFO_LEN;

/**********************************************************************
 * Double Buffer
 *********************************************************************/

type Buffer = [i32; BUFFER_LEN];

#[repr(C)]
#[repr(align(32))]
pub struct DoubleBuffer {
    blks: [ControlBlock; 2],
    bufs: [Buffer; 2],
    amts: [usize; 2],
    chan: usize,
    cur:  bool,
    dma:  DMA,
}

///
///Double buffer for use with the I2S DMA transfer.
///
impl Default for DoubleBuffer {
    fn default() -> Self {
        DoubleBuffer {
            bufs: [[0; BUFFER_LEN]; 2],
            blks: [ControlBlock::default(); 2],
            amts: [0; 2],
            chan: 0,
            cur: false,
            dma: DMA::default(),
        }
    }
}

impl DoubleBuffer {
    fn init(&mut self, chan: usize) {
        if chan < 15 {
            self.dma.ENABLE.set (
                self.dma.ENABLE.get() | (1 << chan) as u32
            );
            self.chan = chan;
        } else {
            panic!("Specified channel out of range.");
        }
    }

    fn activate(&mut self) {
        if self.chan < 15 {
            if !self.dma.CHANNELS[self.chan].CS.is_set(CS::ACTIVE) {
                self.dma.CHANNELS[self.chan].CS.modify(CS::DISDEBUG::CLEAR);
                self.dma.CHANNELS[self.chan].CONBLK_AD.write (
                    CONBLK_AD::SCB_ADDR.val (
                        &self.blks[0] as *const ControlBlock as u32
                    )
                );
                self.dma.CHANNELS[self.chan].CS.modify(CS::ACTIVE::SET);
            } else {
                panic!("Specified channel already activated.");
            }
        } else {
            panic!("Specified channel out of range.");
        }
    }

    pub fn print_status(&self) {
        debug::out("CONBLK_AD: ");
        debug::u32hex(self.dma.CHANNELS[self.chan].CONBLK_AD.get() as u32);
        debug::out("\n");

        debug::out("CS: ");
        debug::u32bits(self.dma.CHANNELS[self.chan].CS.get() as u32);
        debug::out("\n");

        debug::out("INT: ");
        debug::u32hex(self.dma.CHANNELS[self.chan].CS.is_set(CS::INT) as u32);
        debug::out("\n");

        debug::out("END: ");
        debug::u32hex(self.dma.CHANNELS[self.chan].CS.is_set(CS::END) as u32);
        debug::out("\n");

        debug::out("ACTIVE: ");
        debug::u32hex(self.dma.CHANNELS[self.chan].CS.is_set(CS::ACTIVE) as u32);
        debug::out("\n");
        
        debug::out("ERROR: ");
        debug::u32hex(self.dma.CHANNELS[self.chan].CS.is_set(CS::ERROR) as u32);
        debug::out("\n");        

        debug::out("DEBUG: ");
        debug::u32bits(self.dma.CHANNELS[self.chan].DEBUG.get() as u32);
        debug::out("\n");
        
        debug::out("TXFR_LEN: ");
        debug::u32hex(self.dma.CHANNELS[self.chan].TXFR_LEN.get() as u32);
        debug::out("\n");

        debug::out("TI BLK 0,1 & DMA: \n");
        debug::u32bits(self.blks[0].TI);
        debug::out("\n");
        debug::u32bits(self.blks[1].TI);
        debug::out("\n");
        debug::u32bits(self.dma.CHANNELS[self.chan].TI.get() as u32);
        debug::out("\n");

        debug::out("ENABLE: ");
        debug::u32bits(self.dma.ENABLE.get() as u32);
        debug::out("\n");

        debug::out("INT_STATUS: ");
        debug::u32bits(self.dma.INT_STATUS.get() as u32);
        debug::out("\n");
    }
}

#[repr(C)]
#[repr(align(32))]
#[derive(Default)]
///
///Receive buffer.
///
pub struct Rx (DoubleBuffer);

impl Rx {
    pub fn init(&mut self, chan: usize) {

        self.0.init(chan);

        for i in 0..2 {
//TI From Linux: Channel 8.
//0x00030419
//0b0000_0000_0000_0011_0000_0100_0001_1001
//INTEN::SET
//WAIT_RESP
//DEST_INC
//SRC_DREQ
//PCM_RX
//
            self.0.blks[i].TI = (
                TI::PERMAP::PCM_RX + //Use PCM_RX to gate reads.
                TI::SRC_DREQ::SET  + //PCM_RX provides the DREQ.
                TI::DEST_INC::SET  + //Increment destination after each write.
                TI::WAIT_RESP::SET + //? Wait for AXI response for each write.
                TI::INTEN::SET       //Interrupt on completion.
            ).value;

            self.0.blks[i].SOURCE_AD = I2S_FIFO;
            self.0.blks[i].DEST_AD = &self.0.bufs[i] as *const Buffer as u32;
            self.0.blks[i].TXFR_LEN = BUFFER_LEN as u32;
        }

        self.0.blks[0].NEXTCONBK = &self.0.blks[1] as *const ControlBlock as u32;
        self.0.blks[1].NEXTCONBK = &self.0.blks[0] as *const ControlBlock as u32;
        self.0.activate();
    }

    pub fn print_status(&self) {
        debug::out("Rx Status:\n");
        self.0.print_status(); 
    }
}


#[repr(C)]
#[repr(align(32))]
#[derive(Default)]
///
///Transmit buffer.
///
pub struct Tx (DoubleBuffer);

impl Tx {
    pub fn init(&mut self, chan: usize) {
        self.0.init(chan);

        for i in 0..2 {
//TI From Linux: Channel 5.
//0x00020149
//0b0000_0000_0000_0010_0000_0001_0100_1001
//INTEN
//WAIT_RESP
//DEST_DREQ
//SRC_INC
//PCM_TX
//TXFR_LEN 2024
//
            self.0.blks[i].TI = (
                TI::PERMAP::PCM_TX + //2
                TI::SRC_INC::SET   + //Increment source 
                TI::DEST_DREQ::SET + //Use DREQ.
                TI::WAIT_RESP::SET +
                TI::INTEN::SET
            ).value;

            debug::out("Tx.init() blks.TI: ");
            debug::u32bits(self.0.blks[i].TI);
            debug::out("\n");

            self.0.blks[i].SOURCE_AD = &self.0.bufs[i] as *const Buffer as u32;
            self.0.blks[i].DEST_AD = I2S_FIFO;
            self.0.blks[i].TXFR_LEN = BUFFER_LEN as u32;
        }

        self.0.blks[0].NEXTCONBK = &self.0.blks[1] as *const ControlBlock as u32;
        self.0.blks[1].NEXTCONBK = &self.0.blks[0] as *const ControlBlock as u32;
        self.0.activate();
    }

    pub fn print_status(&self) { 
        debug::out("Tx Status:\n");
        self.0.print_status(); 
    }
}


// impl Rx {
//     ///
//     ///Poll rx FIFO and drain when ready.
//     ///
// 
//     pub fn poll(&mut self) {
// //         use cortex_a::{asm};
//         use cortex_a::barrier;
// //         use core::sync::atomic::{compiler_fence, Ordering};
// //         unsafe{ barrier::isb(barrier::SY); }
// //         compiler_fence(Ordering::SeqCst);
//         
//         if self.i2s().CS_A.is_set(CS_A::RXERR) {
//             self.errcnt += 1;
//             self.i2s().CS_A.modify(CS_A::RXERR::SET);
//         }
// 
//         if self.i2s().CS_A.is_set(CS_A::RXR) {
//         unsafe{ barrier::isb(barrier::SY); }
//             while self.i2s().CS_A.is_set(CS_A::RXD) { //Bit 20.
//                 self.queue.enqueue (
//                     ((self.i2s().FIFO_A.get() as i32) as SampleType) / 8388607.0
//                 );
//             }
//         unsafe{ barrier::isb(barrier::SY); }
//         }
//     }
//     
//     #[inline]
//     pub fn ready(&self) -> bool {
//         self.queue.amt() >= rack::unit::PROCESS_BLOCK_LEN
//     }
// 
// ///
// ///Poll rx FIFO and drain. Output a square wave.
// ///
//     #[allow(dead_code)]
//     pub fn test_poll_sq(&mut self) {
//         if self.i2s().CS_A.is_set(CS_A::RXERR) {
//             self.errcnt += 1;
//             self.i2s().CS_A.modify(CS_A::RXERR::SET);
//         }
// 
//         if self.i2s().CS_A.is_set(CS_A::RXR) {
//             while self.i2s().CS_A.is_set(CS_A::RXD) {
//                 self.queue.enqueue (
//                     if (self.recvd % 32) == 0 { -1.0 } else { 1.0 }
//                 );
//                 self.recvd += 1;
//                 self.i2s().FIFO_A.get();
//             }
//         }
//     }
// 
// ///
// ///Poll rx FIFO and drain. Output total number of samples received.
// ///
//     #[allow(dead_code)]
//     pub fn test_poll_cnt(&mut self) {
//         if self.i2s().CS_A.is_set(CS_A::RXERR) {
//             self.errcnt += 1;
//             self.i2s().CS_A.modify(CS_A::RXERR::SET);
//         }
// 
//         if self.i2s().CS_A.is_set(CS_A::RXR) {
//             while self.i2s().CS_A.is_set(CS_A::RXD) {
//                 self.queue.enqueue(self.recvd as SampleType);
//                 self.recvd += 1;
//                 self.i2s().FIFO_A.get();
//             }
//         }
//     }
// }
//
// #[derive(Default)]
// pub struct Tx {
//     pub queue: Queue<SampleType>,
//     pub errcnt: usize,
// }
// 
// impl I2S for Tx {}
// 
// impl Tx {
// ///
// ///Poll tx FIFO and fill when ready.
// ///
//     pub fn poll(&mut self) {
//         if self.i2s().CS_A.is_set(CS_A::TXERR) {
//             self.errcnt += 1;
//             self.i2s().CS_A.modify(CS_A::TXERR::SET);
//         }
// 
//         if self.i2s().CS_A.is_set(CS_A::TXW) {
//             while self.i2s().CS_A.is_set(CS_A::TXD) {
//                 if self.queue.empty_queue() {
//                     break;
//                 } else {
//                     self.i2s().FIFO_A.write(
//                         FIFO_A::DATA.val(
//                             ((self.queue.dequeue() * 8388607.0) as i32) as u32
//                         )
//                     );
//                 }
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
