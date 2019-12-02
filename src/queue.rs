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

///
///Helper module provides initialization and a rx/tx queue for samples
///read from and ready to send to the I2S bus.
///

use peripherals::i2s::*;
use effects::SampleType;
use common::buffer::{Buffer, Read, Write};

trait I2S {
    #[inline]
    fn i2s(&self) -> I2S0 { I2S0::default() }
}


///
///Manages I/O for the I2S bus peripherals.
///
#[derive(Default)]
pub struct Queue {
    q_a: Buffer<SampleType>,
    q_b: Buffer<SampleType>,
}

impl Queue {
///
///True if both queues are full.
///
    #[inline]
    pub fn full(&mut self) -> bool { 
        self.q_a.full_queue() && self.q_b.full_queue() 
    }

///
///True if both queues are empty.
///
    #[inline]    
    pub fn empty(&mut self) -> bool { 
        self.q_a.empty_queue() && self.q_b.empty_queue() 
    }

///
///Enqueue channel a/b.
///
    #[inline]
    pub fn enqueue(&mut self, ab: (SampleType, SampleType)) {
        self.q_a.enqueue(ab.0);
        self.q_b.enqueue(ab.1);
    }

///
///Dequeue channel a/b.
///
    #[inline]
    pub fn dequeue(&mut self) -> (SampleType, SampleType) {
        (self.q_a.dequeue(), self.q_b.dequeue())
    }
}

#[derive(Default)]
pub struct Rx {
    pub queue: Buffer<SampleType>,
    pub errcnt: usize,
}

impl I2S for Rx {}

impl Rx {
///
///Poll rx FIFO and drain when ready.
///
    #[inline]
    pub fn poll(&mut self) {
        if self.i2s().CS_A.is_set(CS_A::RXERR) {
            self.errcnt += 1;
            self.i2s().CS_A.modify(CS_A::RXERR::SET);
        }

        if self.i2s().CS_A.is_set(CS_A::RXR) {
            while self.i2s().CS_A.is_set(CS_A::RXD) {
                self.queue.enqueue (
                    ((self.i2s().FIFO_A.get() as i32) as SampleType) / 8388607.0
                );
            }
        }
    }
}

///
///RxTest()
/// Outputs a known good signal instead of input from i2s.
///
pub struct RxTest {
    pub queue: Buffer<SampleType>,
    pub errcnt: usize,
    pub i: usize,
    pub val: SampleType,
}

impl Default for RxTest {
    fn default() -> Self {
        RxTest {
            queue: Buffer::<SampleType>::default(),
            errcnt: 0,
            i: 0,
            val: 1.0,
        }
    }
}

impl I2S for RxTest {}

impl RxTest {
///
///Poll rx FIFO and drain. Output a square wave.
///
    #[inline]
    pub fn poll_sq(&mut self) {
        if self.i2s().CS_A.is_set(CS_A::RXERR) {
            self.errcnt += 1;
            self.i2s().CS_A.modify(CS_A::RXERR::SET);
        }

        if self.i2s().CS_A.is_set(CS_A::RXR) {
            while self.i2s().CS_A.is_set(CS_A::RXD) {
                if (self.i % 32) == 0 { self.val = -self.val; }
                self.queue.enqueue(self.val);
                self.i += 1;
                self.i2s().FIFO_A.get();
            }
        }
    }

///
///Poll rx FIFO and drain. Output total number of samples received.
///
    #[inline]
    pub fn poll_cnt(&mut self) {
        if self.i2s().CS_A.is_set(CS_A::RXERR) {
            self.errcnt += 1;
            self.i2s().CS_A.modify(CS_A::RXERR::SET);
        }

        if self.i2s().CS_A.is_set(CS_A::RXR) {
            while self.i2s().CS_A.is_set(CS_A::RXD) {
                self.queue.enqueue(self.i as SampleType);
                self.i += 1;
                self.i2s().FIFO_A.get();
            }
        }
    }
}

#[derive(Default)]
pub struct Tx {
    pub queue: Buffer<SampleType>,
    pub errcnt: usize,
}

impl I2S for Tx {}

impl Tx {
///
///Poll tx FIFO and fill when ready.
///
    #[inline]
    pub fn poll(&mut self) {
        if self.i2s().CS_A.is_set(CS_A::TXERR) {
            self.errcnt += 1;
            self.i2s().CS_A.modify(CS_A::TXERR::SET);
        }

        if self.i2s().CS_A.is_set(CS_A::TXW) {
            while self.i2s().CS_A.is_set(CS_A::TXD) {
                self.i2s().FIFO_A.write(
                    FIFO_A::DATA.val(
                        ((self.queue.dequeue() * 8388607.0) as i32) as u32
                    )
                );
            }
        }
    }
}
