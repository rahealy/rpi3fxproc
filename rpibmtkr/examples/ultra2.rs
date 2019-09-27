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
    let mut ultra2 = Ultra2::<I2C1, I2S0, Timer1>::default();
    let i2s = I2S0::default();

//Set up Ultra2 hat.
    ultra2.freeze(false).
           pdn_mic(true).   //Power down microphone.
           pdn_adc(true).   //Power down ADC.
           pdn_dac(false).  //Use DAC
           dac_vol_a(0xFF). //Full volume.
           dac_vol_b(0xFF). //Full volume.
           smplrt(48_000);  //48kHz sample rate.

    if let Err(err) = ultra2.init() {
        debug::out("main(): Error ultra2.init() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

    if let Err(err) = ultra2.cfg() {
        debug::out("main(): Error ultra2.cfg() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//Initially fill buffer with zeroes.
    i2s.tx_fill(0x00000000);

//Power up cs4265.
    if let Err(err) = ultra2.power_up() {
        debug::out("main(): Error ultra2.power_up() failed - ");
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//Turn on i2s transmitter.
    i2s.tx_on(true);

    i2s.print_status();

//    debug::out("main(): Output square wave indefinitely.\r\n");
//Write square wave.
    debug::out("main(): Write square wave.\r\n");
    let mut e: u32 = 0;
    for _ in 0..(48_000 / 218) {
//            debug::out("main(): HERE2!\r\n");
        e = i2s.tx_write_val(0x00080000, 109);
        if e > 0 {
            debug::out("!"); //Error detected.
//            debug::u32bits(e);
        }
//            debug::out("main(): HERE3!\r\n");
        e = i2s.tx_write_val(0x00080001, 109);
        if e > 0 {
            debug::out("!"); //Error detected.
//            debug::u32bits(e);
        }
//            debug::out("main(): HERE4!\r\n");
    }
    debug::out("main(): Square wave written.\r\n");

    loop {
//        debug::out(".");
    }
}
