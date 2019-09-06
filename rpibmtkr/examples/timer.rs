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
 * The RPi has four hardware timers. Two are used by the GPU leaving two for
 * general purpose use. This example demonstrates how to use the timer
 * routines.
 *
 */
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use peripherals::PERIPHERALS;


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
    PERIPHERALS.init();  
    PERIPHERALS.uart.puts("Timer Example.\r\n");

    loop {
//Exercise timer 1.
        PERIPHERALS.uart.puts("Timer1: Begin one second one shot delay.\r\n");
        if let Err(err) = PERIPHERALS.timer.one_shot(1, 1_000_000) {
            PERIPHERALS.uart.puts(err.msg());
        } 
        PERIPHERALS.uart.puts("Timer1: End one second one shot delay.\r\n");
//Exercise timer 3.
        PERIPHERALS.uart.puts("Timer3: Begin one second one shot delay.\r\n");
        if let Err(err) = PERIPHERALS.timer.one_shot(3, 1_000_000) {
            PERIPHERALS.uart.puts(err.msg());
        }
        PERIPHERALS.uart.puts("Timer3: End one second one shot delay.\r\n");
    }
}
