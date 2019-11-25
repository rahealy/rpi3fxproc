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
 */

use crate::uart;

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
        _    => '.'
    }
}

pub fn out(val: &str) {
    let uart = uart::Uart0::default();
    uart.puts(val);
}

pub fn bit(val: bool) {
    let uart = uart::Uart0::default();
    uart.send( if val { '1' } else { '0' } );
}

pub fn u8bitsi(val: u8) {
    let uart = uart::Uart0::default();
    for i in (0..8).rev() {
        uart.send( 
            if ((1 << i) & val) > 0 { '1' } else { '0' } 
        );
    }
}

pub fn u8bits(val: u8) {
    let uart = uart::Uart0::default();
    uart.puts("0b");
    for i in (0..8).rev() {
        if (i % 4) == 0 { uart.send('_'); }
        uart.send( 
            if ((1 << i) & val) > 0 { '1' } else { '0' } 
        );
    }
}

pub fn u8hexi(val: u8) {
    let uart = uart::Uart0::default();
    uart.send(tohex(val >> 4));
    uart.send(tohex(val));
}

pub fn u8hex(val: u8) {
    let uart = uart::Uart0::default();
    uart.puts("0x");
    uart.send(tohex(val >> 4));
    uart.send(tohex(val));
}

pub fn u32bitsi(val: u32) {
    let uart = uart::Uart0::default();
    for i in (0..32).rev() {
        uart.send(
            if ((1 << i) & val) > 0 { '1' } else { '0' } 
        );
    }
}

pub fn u32bits(val: u32) {
    let uart = uart::Uart0::default();
    uart.puts("0b");
    for i in (0..32).rev() {
        if (i < 31) && (i % 4 == 3) { uart.send('_'); }
        uart.send(
            if ((1 << i) & val) > 0 { '1' } else { '0' } 
        );
    }
}

pub fn u32hex(val: u32) {
    let uart = uart::Uart0::default();
    uart.puts("0x");
    for i in (0..8).rev() {
        uart.send (
            tohex((val >> i * 4) as u8)
        );
    }
}

pub fn u64hex(val: u64) {
    let uart = uart::Uart0::default();
    uart.puts("0x");
    for i in (0..16).rev() {
        uart.send (
            tohex((val >> i * 4) as u8)
        );
    }
}
