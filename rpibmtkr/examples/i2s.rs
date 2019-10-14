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
 * Test/Debug the i2s interface.
 */
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use peripherals::{
    debug, 
    uart::Uart0,
    i2s::{I2S, I2S0, Params},
};

#[allow(unused_imports)]
use startup; //Pull in startup code.

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
    I2S0::init();

//Write a bunch of dots to mark boot.
    debug::out("\r\n");
    for _ in 0..72 { debug::out(".") }
    debug::out("\r\n");

//Allocate structures.
    let i2s     = I2S0::default();      //I2S accessor.
    let mut pcm = Params::default(); //Parameter structure to configure the 
                                        //Broadcom PCM peripheral which implements i2s. 

//Configure the parameters.
    pcm.rxon(true).        //PCM will receive data.
        txon(true).        //PCM will transmit data.
        fs_master(true).   //PCM is frame select master.
        clk_master(true).  //PCM is clock master.
        chlen(32,32).      //Each channel will be 32 bits for a frame length of 64 bits.
        smplrt(8_000);     //Sample rate will be 8kHz

    pcm.rx.ch1.enable(true).
                width(24). //Sample width is 24 bits.
                pos(1);    //Sample data starts 1 clock after frame begins.

    pcm.rx.ch2.enable(true).
                width(24). //Sample width is 24 bits.
                pos(33);   //Sample data starts 33 clocks after frame begins.

    pcm.tx.ch1.enable(true).
                width(24). //Sample width is 24 bits.
                pos(1);    //Sample data starts 1 clock after frame begins.

    pcm.tx.ch2.enable(true).
                width(24). //Sample width is 24 bits.
                pos(33);   //Sample data starts 33 clocks after frame begins.

//Load configuration.
    i2s.load(&pcm);

//Per datasheet fill the FIFO before enabling transmit.
    i2s.tx_fill(0x00FAFAFA);

//Print the state of the PCM status bits.
    i2s.print_status();
    
    
    loop {
    }
}
