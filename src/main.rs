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

#![no_std]
#![no_main]
#![feature(global_asm)]



#[allow(unused_imports)]
use startup; //Pull in the startup code for linking.

use cortex_a;
use core::panic::PanicInfo;
use hats::ultra2::Ultra2;
use peripherals::debug;
use peripherals::i2c::{I2C, I2C1};
use peripherals::i2s::{I2S, I2S0, CS_A, FIFO_A};
use peripherals::timer::{Timer1};
use peripherals::uart::Uart0;

/// 
/// Rust requires a panic handler. On panic go into an infinite loop.
///
/// #Arguments
///
/// * `_info` - Unused. Required by the rust panic handler function spec.
///
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} } 

fn init_ultra2() {
//Ultra2 uses a cs4265 which relies on i2c bus for control. Use RPi I2C1.
//CS4265 communicates audio data via i2s. Use RPi I2S0.
//Various Ultra2 operations requre a delay so use RPi System Timer1
    let mut ultra2 = Ultra2::<I2C1, I2S0, Timer1>::default();
    let i2s = I2S0::default();

//Set up Ultra2 hat.
    ultra2.pdn_mic(true).   //Power down microphone.
           pdn_adc(true).   //Don't use ADC.
           pdn_dac(false).  //Use DAC
           dac_vol_a(0xFF). //Full volume.
           dac_vol_b(0xFF). //Full volume.
           smplrt(48_000);  //48kHz sample rate.

    if let Err(err) = ultra2.init() {
        debug::out("rpi3fxproc::init_ultra2(): Error ultra2.init() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

    if let Err(err) = ultra2.cfg() {
        debug::out("rpi3fxproc::init_ultra2(): Error ultra2.cfg() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//Initially fill buffer with zeroes.
    debug::out("rpi3fxproc::init_ultra2(): Zero transmit buffer.\r\n");
    i2s.tx_fill(0x00000000);


//Turn on i2s transmitter.
    debug::out("rpi3fxproc::init_ultra2(): Turn on RPi i2s transmitter.\r\n");
    i2s.tx_on(true);

//Power up ultra2.
    debug::out("rpi3fxproc::init_ultra2(): Power up Ultra2.\r\n");
    if let Err(err) = ultra2.power_up() {
        debug::out("rpi3fxproc::init_ultra2: Error ultra2.power_up() failed - ");
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

    let mut i: u64 = 0;
    let mut run = true;
    let mut txerr: u64 = 0;
    let mut val: i32 = 0x007F_FFFF;

    debug::out("rpi3fxproc::init_ultra2(): Write 1 second of data.\r\n");
    while run {
        let cs = i2s.CS_A.extract();

        if cs.is_set(CS_A::TXERR) {
            txerr += 1;
            i2s.CS_A.modify(CS_A::TXERR::SET);
        }

        if cs.is_set(CS_A::TXW) {
            while i2s.CS_A.is_set(CS_A::TXD) {
                i += 1;
                if i < 96000 {
                    if (i % 32) == 0 { val = -val; }
                    i2s.FIFO_A.write(FIFO_A::DATA.val(val as u32));
                } else {
                    run = false;
                    break;
                }
            }
        }
    }

    debug::out("rpi3fxproc::init_ultra2(): Results:\r\n");
    debug::out("txerr: ");
    debug::u64hex(txerr);
    debug::out("\r\n");

    debug::out("rpi3fxproc::init_ultra2(): RPi i2s status:\r\n");
    i2s.print_status();
}

fn print_splash() {
    debug::out("\r\n");
    for _ in 0..72 { debug::out(".") }
    debug::out("\r\n");
    debug::out("rpi3fxproc - RaspberryPi 3 Bare Metal Effects Processor\r\n");
    for _ in 0..72 { debug::out(".") }
    debug::out("\r\n");
}

///
/// Main loop.
///
#[export_name = "main"] //So startup.rs can find fn main().
fn main() -> ! {
    Uart0::init();
    I2C1::init();
    I2S0::init(); 

    print_splash();
//     timer_test();
//     exception_test();

    init_ultra2();

    loop{
        cortex_a::asm::wfe();
    }
}

// fn timer_test() {
//     use peripherals::timer::Timer;
//     let t1 = Timer1::default();
// 
//     debug::out("Timer test begin.\r\n");
//     t1.one_shot(1_000_000);
//     debug::out("Timer test end.\r\n");
// }
// 
// fn exception_test() {
//     debug::out("Raise exception.\r\n");
//     let big_addr: u64 = 3 * 1024 * 1024 * 1024;
//     unsafe { core::ptr::read_volatile(big_addr as *mut u64) };
//     debug::out("Exception return.\r\n");
// }
