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

use peripherals::i2s::*;
use rack::effect::SampleType;
use common::buffer::{Queue, Read, Write, Amount};
use peripherals::debug;

trait I2S {
    fn i2s(&self) -> I2S0 { I2S0::default() }
}

#[derive(Default)]
pub struct Rx {
    pub queue: Queue<SampleType>,
    pub errcnt: usize,
    pub recvd: usize,
}

impl I2S for Rx {}

impl Rx {
    ///
    ///Poll rx FIFO and drain when ready.
    ///

    pub fn poll(&mut self) {
//         use cortex_a::{asm};
        use cortex_a::barrier;
//         use core::sync::atomic::{compiler_fence, Ordering};
//         unsafe{ barrier::isb(barrier::SY); }
//         compiler_fence(Ordering::SeqCst);
        
        if self.i2s().CS_A.is_set(CS_A::RXERR) {
            self.errcnt += 1;
            self.i2s().CS_A.modify(CS_A::RXERR::SET);
        }

        if self.i2s().CS_A.is_set(CS_A::RXR) {
        unsafe{ barrier::isb(barrier::SY); }
            while self.i2s().CS_A.is_set(CS_A::RXD) { //Bit 20.
                self.queue.enqueue (
                    ((self.i2s().FIFO_A.get() as i32) as SampleType) / 8388607.0
                );
            }
        unsafe{ barrier::isb(barrier::SY); }
        }
    }
    
    #[inline]
    pub fn ready(&self) -> bool {
        self.queue.amt() >= rack::unit::PROCESS_BLOCK_LEN
    }

///
///Poll rx FIFO and drain. Output a square wave.
///
    #[allow(dead_code)]
    pub fn test_poll_sq(&mut self) {
        if self.i2s().CS_A.is_set(CS_A::RXERR) {
            self.errcnt += 1;
            self.i2s().CS_A.modify(CS_A::RXERR::SET);
        }

        if self.i2s().CS_A.is_set(CS_A::RXR) {
            while self.i2s().CS_A.is_set(CS_A::RXD) {
                self.queue.enqueue (
                    if (self.recvd % 32) == 0 { -1.0 } else { 1.0 }
                );
                self.recvd += 1;
                self.i2s().FIFO_A.get();
            }
        }
    }

///
///Poll rx FIFO and drain. Output total number of samples received.
///
    #[allow(dead_code)]
    pub fn test_poll_cnt(&mut self) {
        if self.i2s().CS_A.is_set(CS_A::RXERR) {
            self.errcnt += 1;
            self.i2s().CS_A.modify(CS_A::RXERR::SET);
        }

        if self.i2s().CS_A.is_set(CS_A::RXR) {
            while self.i2s().CS_A.is_set(CS_A::RXD) {
                self.queue.enqueue(self.recvd as SampleType);
                self.recvd += 1;
                self.i2s().FIFO_A.get();
            }
        }
    }
}


#[derive(Default)]
pub struct Tx {
    pub queue: Queue<SampleType>,
    pub errcnt: usize,
}

impl I2S for Tx {}

impl Tx {
///
///Poll tx FIFO and fill when ready.
///
    pub fn poll(&mut self) {
        if self.i2s().CS_A.is_set(CS_A::TXERR) {
            self.errcnt += 1;
            self.i2s().CS_A.modify(CS_A::TXERR::SET);
        }

        if self.i2s().CS_A.is_set(CS_A::TXW) {
            while self.i2s().CS_A.is_set(CS_A::TXD) {
                if self.queue.empty_queue() {
                    break;
                } else {
                    self.i2s().FIFO_A.write(
                        FIFO_A::DATA.val(
                            ((self.queue.dequeue() * 8388607.0) as i32) as u32
                        )
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
