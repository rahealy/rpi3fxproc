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
#![feature(alloc_error_handler)]

extern crate alloc;

use cortex_a;
use core::panic::PanicInfo;
use hats::ultra2::Ultra2;
use hats::ultra2;
use peripherals::debug;
use peripherals::i2c::*;
use peripherals::i2s::*;
use peripherals::timer::{Timer1};
use peripherals::uart::Uart0;
use peripherals::MMIO_BASE;
use effects::SAMPLE_RATE_USIZE;
use effects::SampleType;
use linked_list_allocator::LockedHeap;

mod queue;
mod rack;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[allow(unused_imports)]
use startup;
use startup::STACK_START;

///
///Heap variables.
///
const HEAP_START: usize = (STACK_START + 8) as usize;
const HEAP_SIZE:  usize = (MMIO_BASE as usize) - HEAP_START;

///
/// Rust requires a panic handler. On panic go into an infinite loop.
///
/// #Arguments
///
/// * `_info` - Unused. Required by the rust panic handler function spec.
///
#[panic_handler]
fn panic(info: &PanicInfo) -> ! { 
    debug::out("rpi3fxproc::panic(): Panic encountered: ");

    if let Some(s) = info.payload().downcast_ref::<&str>() {
//FIXME: Not sure if this works.
        debug::out(s);
        debug::out("\r\n");
    }

    debug::out("\r\nHalted.\r\n");
    loop {
        cortex_a::asm::wfe();
    } 
} 

fn init_heap() {
    debug::out("rpi3fxproc::init_heap(): Begin.\r\n");
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    debug::out("rpi3fxproc::init_heap(): End.\r\n");
}


fn init_ultra2() {
//Ultra2 uses a cs4265 which relies on i2c bus for control. Use RPi I2C1.
//CS4265 communicates audio data via i2s. Use RPi I2S0.
//Various Ultra2 operations requre a delay so use RPi System Timer1
    let mut ultra2 = Ultra2::<I2C1, I2S0, Timer1>::default();
    let i2s = I2S0::default();

    debug::out("rpi3fxproc::init_ultra2(): Begin Ultra2 initialization.\r\n");

    if let Err(err) = ultra2.init() {
        debug::out("rpi3fxproc::init_ultra2(): Error ultra2.init() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//Set up Ultra2 hat.
    let mut u2params = ultra2::Params::default();

    u2params.adc_gain_a(0).   //Unity Gain.
             adc_gain_b(0).   //Unity Gain.
             dac_vol_a(0xFF). //Full volume.
             dac_vol_b(0xFF). //Full volume.
             adc_sel(0).      //Select on-board microphones for adc input.
             pdn_mic(false).  //Power up microphone.
             pdn_adc(false).  //Power up ADC.
             pdn_dac(false).  //Power up DAC
             smplrt(SAMPLE_RATE_USIZE as u32); //48kHz sample rate.

    if let Err(err) = ultra2.load(&u2params) {
        debug::out("rpi3fxproc::init_ultra2(): Error ultra2.load() failed - ");     
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//Initially fill i2s FIFO with zeroes.
    debug::out("rpi3fxproc::init_ultra2(): Zero transmit buffer.\r\n");
    i2s.tx_fill(0x00000000);

//Turn on i2s.
    debug::out("rpi3fxproc::init_ultra2(): Turn on RPi i2s TX and RX.\r\n");
    i2s.tx_on(true);
    i2s.rx_on(true);

//Power up ultra2.
    debug::out("rpi3fxproc::init_ultra2(): Power up Ultra2.\r\n");
    if let Err(err) = ultra2.power_up() {
        debug::out("rpi3fxproc::init_ultra2: Error ultra2.power_up() failed - ");
        debug::out(err.msg());
        debug::out("\r\n");
        panic!();
    }

//     debug::out("rpi3fxproc::init_ultra2(): RPi i2s status:\r\n");
//     i2s.print_status();

    debug::out("rpi3fxproc::init_ultra2(): Initialized.\r\n");
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
    use crate::rack::connection::factory::{From, To, Connect};
    use common::buffer::{Read, Write, Size, Amount};

    Uart0::init();
    I2C1::init();
    I2S0::init();

    print_splash();

    init_heap();
    init_ultra2();

    sound_test();

//    let mut rxt = queue::RxTest::default();
    let mut rx  = queue::Rx::default();
    let mut tx  = queue::Tx::default();
    let mut u0  = rack::Unit::new();

    debug::out("rpi3fxproc::main(): Connecting effects.\r\n");

    let connections: [[usize;3]; 4] = [
        [rack::INPUT_A, 4, 100],  //From Input A to delay0.
        [4, rack::OUTPUT_A, 100], //From delay0 to Output A
        [rack::INPUT_B, 5, 100],  //From Input B to delay1.
        [5, rack::OUTPUT_B, 100], //From delay1 to Output B
    ];

    for conn in connections.iter() {
        if let Err(err) = u0.from(conn[0])        //Input A
                            .to(conn[1], conn[2]) //Process input of delay 0
                            .connect()
        {
            debug::out(err);
            debug::out("\r\n");
        }
    }

    debug::out("rpi3fxproc::main(): Queueing effect processing order.\r\n");

    let queue = [rack::INPUT_A, rack::INPUT_B, 4, 5];

    for effect in queue.iter() {
        if let Err(err) = u0.queue(*effect) {
            debug::out(err);
            debug::out("\r\n");
        }
    }

    debug::out("rpi3fxproc::main(): Begin processing.\r\n");

    loop { 
//Poll i2s and queue samples. Process and queue results. Transmit.
        let mut i = 0;

        while i < SAMPLE_RATE_USIZE * 2 {
            rx.poll();
            if rx.queue.amt() > (rx.queue.size() / 2) {
                i += u0.process(&mut rx.queue, &mut tx.queue);
            }
            tx.poll();
        }

        debug::out("rx errcnt: ");
        debug::u64hex(rx.errcnt as u64);
        debug::out("\r\n");
        rx.errcnt = 0;

        debug::out("tx errcnt: ");
        debug::u64hex(tx.errcnt as u64);
        debug::out("\r\n");
        tx.errcnt = 0;
    }
}

///
///Implement a basic 2 second delay.
///
#[allow(dead_code)]
fn echo_test() {
    use common::offset::Offset;

    debug::out("rpi3fxproc::echo_test(): Begin.\r\n");

    const DELAY:        usize = SAMPLE_RATE_USIZE; //2 second delay.
    const NUM_CHANNELS: usize = 2;
    const DELAY_BUF_SZ: usize = DELAY * NUM_CHANNELS * 2;
    const FEEDBACK:     i32   = 3;
    const WET:          i32   = 2;
    const DRY:          i32   = 2;
    const NUM_SAMPLES: usize  = DELAY_BUF_SZ * 3; //Process 6 seconds then report.

    let i2s            = I2S0::default();
    let mut i: usize   = NUM_SAMPLES;
    let mut rxerr: u64 = 0;
    let mut txerr: u64 = 0;
    let mut rd: Offset = Offset(0);
    let mut wr: Offset = Offset(DELAY * NUM_CHANNELS);
    let mut buf        = [0 as i32; DELAY_BUF_SZ];
    let mut val: i32   = 0x007FFFFF;

    loop {
        while i > 0 {
            if i2s.CS_A.is_set(CS_A::RXERR) {
                rxerr += 1;
                i2s.CS_A.modify(CS_A::RXERR::SET);
            }

            if i2s.CS_A.is_set(CS_A::TXERR) {
                txerr += 1;
                i2s.CS_A.modify(CS_A::TXERR::SET);
            }

            if i2s.CS_A.is_set(CS_A::RXR) {
                if i2s.CS_A.is_set(CS_A::TXW) {
                    let smpl_rd  = buf[rd.0];
                    let smpl_wr  = buf[wr.0];
                    let smpl_in  = i2s.FIFO_A.get() as i32;
                    let smpl_out = (smpl_in / DRY) + (smpl_rd / WET);

                    i2s.FIFO_A.write(FIFO_A::DATA.val(smpl_out as u32));
                    buf[wr.0] = smpl_in + (smpl_wr / FEEDBACK);

                    wr.inc(DELAY_BUF_SZ);
                    rd.inc(DELAY_BUF_SZ);
                    i -= 1;
                }
            }
        }

        debug::out("txerr: ");
        debug::u64hex(txerr);
        debug::out("\r\n");
        txerr = 0;

        debug::out("rxerr: ");
        debug::u64hex(rxerr);
        debug::out("\r\n");
        rxerr = 0;

        i = NUM_SAMPLES;
    }
}

///
///Write a 1 second square wave to the I2S peripheral.
///
#[allow(dead_code)]
fn sound_test() {
    let i2s = I2S0::default();
    let mut i: usize = 0;
    let mut run = true;
    let mut txerr: u64 = 0;
    let mut val: i32 = 0x007FFFFF;

    debug::out("rpi3fxproc::sound_test(): Begin.\r\n");

    while run {
        let cs = i2s.CS_A.extract();

        if cs.is_set(CS_A::TXERR) {
            txerr += 1;
            i2s.CS_A.modify(CS_A::TXERR::SET);
        }

        if cs.is_set(CS_A::TXW) {
            while i2s.CS_A.is_set(CS_A::TXD) {
                i += 1;
                if i < (SAMPLE_RATE_USIZE * 2) {
                    if (i % 32) == 0 { val = -val; }
                    i2s.FIFO_A.write(FIFO_A::DATA.val(val as u32));
                } else {
                    run = false;
                    break;
                }
            }
        }
    }

    debug::out("txerr: ");
    debug::u64hex(txerr);
    debug::out("\r\n");

    debug::out("rpi3fxproc::sound_test(): End.\r\n");
}
