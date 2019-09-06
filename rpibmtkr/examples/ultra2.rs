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
use peripherals::PERIPHERALS;
//use drivers::cs4265;
use hats::ultra2;

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
/// Main loop exercises timers.
///
#[export_name = "main"] //So startup.rs can find fn main().
fn main() -> ! {
    let mut ultra2 = ultra2::Ultra2::new();

    PERIPHERALS.init();
    PERIPHERALS.uart.puts("Ultra2 Example.\r\n");

    if let Err(err) = ultra2.init() {
        PERIPHERALS.uart.puts("main(): Error ultra2.init() failed - ");        
        PERIPHERALS.uart.puts(err.msg());
        PERIPHERALS.uart.puts("\r\n");
        panic!();
    }

    loop {
    }
}
