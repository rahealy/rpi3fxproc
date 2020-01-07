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
use peripherals::dma;
use peripherals::dma::*;
use peripherals::debug;
//use rack::effect::SampleType;
//use common::buffer::{Queue, Read, Write, Amount};
//use common::buffer::{Queue};

const BUFFER_LEN: usize = i2s::FIFO_LEN;

/**********************************************************************
 * Double Buffer
 *********************************************************************/

type Buffer = [i32; BUFFER_LEN];

#[repr(C)]
#[repr(align(32))]
pub struct DoubleBuffer {
    blks: [ControlBlockInstance; 2],
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
            blks: [ControlBlockInstance::default(); 2],
            amts: [0; 2],
            chan: 0,
            cur: false,
            dma: DMA::default(),
        }
    }
}

impl DoubleBuffer {
    ///
    ///Memory location of the DMA Control block as a u32.
    ///
    fn blkloc(&self, idx: usize) -> u32 {
        &self.blks[idx] as *const ControlBlockInstance as u32
    }

    ///
    ///Memory location of the buffer as a u32.
    ///
    fn bufloc(&self, idx: usize) -> u32 {
        &(self.bufs[idx]) as *const Buffer as u32
    }

    ///
    ///Set DMA channel this buffer will use.
    ///
    fn init(&mut self, chan: usize) {
        //Sanity check for required DMA control block alignment.
        if self.blkloc(0) % 32 != 0 { panic!(); }
        if self.blkloc(1) % 32 != 0 { panic!(); }

        if chan < 15 {
            self.chan = chan;
            self.blks[0].NEXTCONBK.set(dma::phy_mem_to_vc_loc(self.blkloc(1)));
            self.blks[1].NEXTCONBK.set(dma::phy_mem_to_vc_loc(self.blkloc(0)));
            self.blks[0].TXFR_LEN.set(BUFFER_LEN as u32);
            self.blks[1].TXFR_LEN.set(BUFFER_LEN as u32);
        } else {
            panic!("Specified channel out of range.");
        }
    }

    fn activate(&mut self) {
        use cortex_a::asm;

//FIXME: Is memory barrier necessary?
//        unsafe { cortex_a::barrier::dmb(cortex_a::barrier::SY); }

        self.dma.ENABLE.set ( 
            self.dma.ENABLE.get() | (1 << self.chan)
        );

        self.dma.CHANNELS[self.chan].CS.modify ( CS::RESET::SET );

        self.dma.CHANNELS[self.chan].CONBLK_AD.write ( 
            CONBLK_AD::SCB_ADDR.val (dma::phy_mem_to_vc_loc(self.blkloc(0)))
        );

        self.dma.CHANNELS[self.chan].CS.modify(
            CS::WAIT_FOR_OUTSTANDING_WRITES::SET + //Finish transfer before moving to next.
            CS::PANIC_PRIORITY.val(15)           + //? Because Circle does this.
            CS::PRIORITY.val(1)                  + //? Because Circle does this too.
            CS::ACTIVE::SET                        //Away we go!
        );

//FIXME: Is memory barrier necessary?
//        unsafe { cortex_a::barrier::dmb(cortex_a::barrier::SY); }
    }

    pub fn print_debug(&self) {
        debug::out("Block 0 (");
        debug::u32hex(self.blkloc(0));
        debug::out("): \n");
        unsafe {
            for i in 0..8 {
                let blk0 = (self.blkloc(0) + (4 * i)) as *const u32;
                debug::u32hex(*blk0);
                debug::out(":");
                debug::u32bits(*blk0);
                debug::out("\n");
            }
        }
        self.blks[0].print_debug();
        
        debug::out("Block 1 (");
        debug::u32hex(self.blkloc(1));
        debug::out("): \n");
        self.blks[1].print_debug();

        debug::out("\n");
        debug::out("DMA Channel ");
        debug::u32hex(self.chan as u32);
        debug::out(":\n");
        
        self.dma.CHANNELS[self.chan].print_debug();
        self.dma.INT_STATUS.print_debug();
        self.dma.ENABLE.print_debug();
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

///
///Initialize and activate i2s Rx dma buffer.
///
///TI From Linux: Channel 8.
///0x00030419
///0b0000_0000_0000_0011_0000_0100_0001_1001
///INTEN::SET
///WAIT_RESP
///DEST_INC
///SRC_DREQ
///PCM_RX
///
    pub fn activate(&mut self, chan: usize) {
        self.0.init(chan);

        for i in 0..2 {
            self.0.blks[i].TI.write (
                TI::PERMAP::PCM_RX + //Use PCM_RX to gate reads.
                TI::SRC_DREQ::SET  + //PCM_RX provides the DREQ.
                TI::DEST_INC::SET  + //Increment destination after each write.
                TI::WAIT_RESP::SET + //? Wait for AXI response for each write.
                TI::INTEN::SET       //Interrupt on completion.
            );

            self.0.blks[i].SOURCE_AD.set (
                dma::mmio_to_vc_loc(i2s::PCM_FIFO)
            );
            
            self.0.blks[i].DEST_AD.set (
                dma::phy_mem_to_vc_loc(self.0.bufloc(i))
            );
        }

        self.0.activate();
    }

    pub fn print_status(&self) {
        debug::out("\nRx Status:\n");
        self.0.print_debug();
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

///
///Initialize and activate i2s Tx dma buffer.
///
///TI From Linux: Channel 5.
///0x00020149
///0b0000_0000_0000_0010_0000_0001_0100_1001
///INTEN
///WAIT_RESP
///DEST_DREQ
///SRC_INC
///PCM_TX
///TXFR_LEN 2024
///
    pub fn activate(&mut self, chan: usize) {
        self.0.init(chan);

        for i in 0..2 {
            self.0.blks[i].TI.write (
                TI::PERMAP::PCM_TX + //Use PCM_TX to gate reads.
                TI::SRC_INC::SET   + //Increment source after each read.
                TI::DEST_DREQ::SET + //PCM_TX provides the DREQ.
                TI::WAIT_RESP::SET + //? Wait for AXI response for each write.
                TI::INTEN::SET       //Interrupt on completion.
            );

            self.0.blks[i].SOURCE_AD.set(
                dma::phy_mem_to_vc_loc(self.0.bufloc(i))
            );

            self.0.blks[i].DEST_AD.set(
                dma::mmio_to_vc_loc(i2s::PCM_FIFO)
            );
        }
        
        debug::out("i2squeue::Tx::activate(): Configured but not activated\n");
        self.print_status();
        
        self.0.activate();
    }

    pub fn print_status(&self) { 
        debug::out("\nTx Status:\n");
        self.0.print_debug(); 
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
