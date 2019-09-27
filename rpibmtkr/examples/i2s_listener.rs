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
 * This program changes I2S pins to inputs and analyzes the input from an i2s
 * device (ultra2 in this case) acting as master.
 */
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use peripherals::{
    debug, 
    uart::Uart0,
    i2s::{I2S0},
    i2c::{I2C, I2C1},
    gpfsel::{GPFSEL,GPFSEL1,GPFSEL2},
    gplev::{GPLEV, GPLEV0},
    timer::{Timer1}
};
use drivers::cs4265;
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

fn read_i2s_frame() {
    let gplev = GPLEV::default();
    let bit = gplev.GPLEV0.is_set(GPLEV0::LEV18);

    debug::out("read_i2s_frame(): Poll for bit flip on GPIO18, I/O Pin 12 (BCLK).\r\n");  
    debug::bit(bit);
    debug::out("\r\n");

    loop {
        if gplev.GPLEV0.is_set(GPLEV0::LEV18) != bit {
//            bit = !bit;
            break;
        }
    }
    debug::out("read_i2s_frame(): Bit flipped.\r\n");  
}

///
/// Main loop.
///
#[export_name = "main"] //So startup.rs can find fn main().
fn main() -> ! {
    Uart0::init();
    debug::init();
    I2C1::init();
    let gpfs = GPFSEL::default();

//Write a bunch of dots to mark boot.
    debug::out("\r\n");
    for _ in 0..72 { debug::out(".") }
    debug::out("\r\n");

//Set up I/O pins for reading I2S0.
    gpfs.GPFSEL1.modify(GPFSEL1::FSEL18::INPUT); //PCM_CLK    
    gpfs.GPFSEL1.modify(GPFSEL1::FSEL19::INPUT); //PCM_FS
    gpfs.GPFSEL2.modify(GPFSEL2::FSEL20::INPUT); //PCM_DIN
    gpfs.GPFSEL2.modify(GPFSEL2::FSEL21::INPUT); //PCM_DOUT
    
//Ultra2 uses a cs4265 which relies on i2c bus for control. Use RPi I2C1.
//CS4265 communicates audio data via i2s. Use RPi I2S0.
//Various Ultra2 operations requre a delay so use RPi System Timer1
    let mut ultra2 = Ultra2::<I2C1, I2S0, Timer1>::default();

//Init only initializes the ultra2 board and doesn't touch the RPi I2S peripheral.
    debug::out("main(): Initialize Ultra2.\r\n");
    if let Err(err) = ultra2.init() {
        debug::out("main(): Error ultra2.init() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//Set up Ultra2 hat.
    ultra2.freeze(false).
           pdn_mic(false).   //Power down microphone.
           pdn_adc(false).   //Power down ADC.
           pdn_dac(false).  //Use DAC
           dac_vol_a(0xFF). //Full volume.
           dac_vol_b(0xFF). //Full volume.
           smplrt(48_000);  //48kHz sample rate.

//Set DAC control.
    debug::out("main(): Set Ultra2 DAC.\r\n");
    ultra2.cs4265.reg.DACCTL1.modify (
        cs4265::DACCTL1::DAC_DIF::I2S24BIT + //Use I2S protocol.
        cs4265::DACCTL1::DEEMPH::CLEAR     + //No de-emphaisis.
        cs4265::DACCTL1::MUTEDAC::CLEAR      //Unmuted.
    );

//Set clock for sample rate and Ultra2 board's clock rate (12.288 MHz).
    debug::out("main(): Set Ultra2 Clock.\r\n");     
    if let Err(err) =  ultra2.cs4265.modify_clk(48_000, 12_288_000) {
        debug::out("main(): Error ultra2.cfg() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//Load registers.
    debug::out("main(): Load Ultra2 registers.\r\n");    
    if let Err(err) =  ultra2.cs4265.ld_regs() {
        debug::out("main(): Error ultra2.ld_regs() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//Print status of CS4265.
    if let Err(err) =  ultra2.cs4265.print_status() {
        debug::out("main(): Error ultra2.print_status() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//Verify local copy of registers matches CS4265 registers.
    if let Err(err) =  ultra2.cs4265.verify_regs() {
        debug::out("main(): Error ultra2.verify_regs() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//Power up cs4265.
    debug::out("main(): Power up Ultra2.\r\n");     
    if let Err(err) = ultra2.power_up() {
        debug::out("main(): Error ultra2.power_up() failed - ");
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

    debug::out("main(): reading one i2s frame.\r\n");

    read_i2s_frame();

    loop {
    }
}
