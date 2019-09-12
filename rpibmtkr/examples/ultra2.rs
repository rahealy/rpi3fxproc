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
 * Use the Ultra2 RPi hat.
 */
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use peripherals::{
    debug, 
    uart::Uart0,
    i2s::{I2S, I2S0},
    i2c::{I2C, I2C1},
    timer::{Timer1}
};
use hats::ultra2::Ultra2;

mod startup; //Pull in startup code.

/// 
/// Rust requires a panic handler. On panic go into an infinite loop.
///
/// #Arguments
///
/// * `_info` - Unused. Required by the rust panic handler function spec.
///
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }


///
/// Main loop.
///
#[export_name = "main"] //So startup.rs can find fn main().
fn main() -> ! {
    Uart0::init();
    debug::init();
    I2C1::init();
    I2S0::init();

    
//Write a bunch of dots to mark boot.
    debug::out("\r\n");
    for _ in 0..72 { debug::out(".") }
    debug::out("\r\n");
    
//Ultra2 uses a cs4265 which relies on i2c bus for control. Use RPi I2C1.
//CS4265 communicates audio data via i2s. Use RPi I2S0.
//Various Ultra2 operations requre a delay so use RPi System Timer1
    let mut u2 = Ultra2::<I2C1, I2S0, Timer1>::default();

    if let Err(err) = u2.init() {
        debug::out("main(): Error ultra2.init() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

    loop {
    }
}
