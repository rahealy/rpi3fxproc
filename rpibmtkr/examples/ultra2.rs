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
    i2s,
    uart::Uart0,
    i2c::{ I2C, I2C1 },
    timer::{ Timer1 }
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

//Ultra2 uses a cs4265 which relies on i2c bus for control. Use RPi I2C1.
//Various Ultra2 operations requre a delay so use RPi System Timer1
    let mut u2 = Ultra2::<I2C1, Timer1>::default();

    if let Err(err) = u2.init() {
        debug::out("main(): Error ultra2.init() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//The cs425 uses i2s to communicate audio data to the RPi.
//Set up RPi i2s as slave for 2 channel 48kHz, 16bit audio. 
    let mut i2s_parms = i2s::PCMParams::default();

    i2s_parms.rxon(true).
              txon(true).
              fs_master(false).
              clk_master(false).
              chlen(16,16);

    i2s_parms.rx.ch1.enable(true).
                     width(16).
                     pos(1);
                     
    i2s_parms.rx.ch2.enable(true).
                     width(16).
                     pos(17);
                     
    i2s_parms.tx.ch1.enable(true).
                     width(16).
                     pos(1);
                     
    i2s_parms.tx.ch2.enable(true).
                     width(16).
                     pos(17);
 
    i2s::I2S::default().init(&i2s_parms);

    loop {
    }
}
