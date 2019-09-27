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
 * Writes informational text to uart0.
 * FIXME: Get rid of singleton-like construction. Rust doesn't like it.
 */

use crate::uart;

pub struct Dbg {
    uart: uart::Uart0
}

pub fn init() {
    unsafe {
        DBG = Some(
            Dbg {
                uart: uart::Uart0
            }
        );
    }
}

pub fn out(val: &str) {
    unsafe {
        match &mut DBG {
            Some(dbg) => dbg.uart.puts(val),
            None => {}
        }
    }
}

pub fn bit(val: bool) {
    unsafe {
        match &mut DBG {
            Some(dbg) => {
                dbg.uart.send( if val { '1' } else { '0' } );
            }
            None => {}
        }
    }    
}

pub fn u8bits(val: u8) {
    unsafe {
        match &mut DBG {
            Some(dbg) => {
                for i in (0..8).rev() {
                    dbg.uart.send( 
                        if ((1 << i) & val) > 0 { '1' } else { '0' } 
                    );
                }
            }
            None => {}
        }
    }    
}

pub fn u32bits(val: u32) {
    unsafe {
        match &mut DBG {
            Some(dbg) => {
                for i in (0..32).rev() {
                    dbg.uart.send( 
                        if ((1 << i) & val) > 0 { '1' } else { '0' } 
                    );
                }
            }
            None => {}
        }
    }    
}

pub static mut DBG: Option<Dbg> = None;
