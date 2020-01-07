/*
 * MIT License
 *
 * Copyright (c) 2019 Richard Healy
 *
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
#![feature(asm)]
#![feature(global_asm)]

///
///The stack spans between end of read only code area and 4MB boundary.
///
pub const STACK_START: u64 = 0x0080_0000; //Stack decrements from 8MB boundary.
pub const MMIO_BASE: usize = 0x3F00_0000; //Peripheral access starts at 1GB boundary.

///
///Heap starts at base of stack with an 8 byte buffer between the two.
///FIXME: For now use the whole memory space. multi-core will come later.
///
pub const HEAP_START: usize = STACK_START as usize; //(STACK_START + 8) as usize;
pub const HEAP_SIZE:  usize = 0x0080_0000; //MMIO_BASE - HEAP_START;

mod uart;
mod mbox;
mod memory;
mod exceptions;
mod startup;
