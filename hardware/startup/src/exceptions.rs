/*
 * MIT License
 *
 * Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
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

use cortex_a::{barrier, regs::*};
use crate::uart::Uart0;

global_asm!(include_str!("vectors.S"));


/// The default exception, invoked for every exception type unless the handler
/// is overwritten.
#[no_mangle]
unsafe extern "C" fn default_exception_handler() {
    Uart0::default().puts("Unexpected exception.\r\n");
    loop {
        cortex_a::asm::wfe()
    }
}


pub fn init() {
    let uart = Uart0::default();
    uart.puts("exceptions::init(): Setting up exception table.\r\n"); 

    extern "C" {
        static __exception_vectors_start: u64;
    }

    unsafe {
        let exception_vectors_start: u64 = &__exception_vectors_start as *const _ as u64;
        uart.puts("exceptions::init(): Vector table starts at "); 
        uart.u64hex(exception_vectors_start);
        uart.puts("\r\n");

        barrier::isb(barrier::SY);
        cortex_a::regs::VBAR_EL1.set(exception_vectors_start);
        barrier::isb(barrier::SY);
    }

    uart.puts("exceptions::init(): Done setting up exception table.\r\n");
    return;
}


#[repr(C)]
pub struct GPR {
    x: [u64; 31],
}

#[repr(C)]
pub struct ExceptionContext {
    // General Purpose Registers
    gpr: GPR,
    spsr_el1: u64,
    elr_el1: u64,
}

// To implement an exception handler, overwrite it by defining the respective
// function below.
// Don't forget the #[no_mangle] attribute.
//
// unsafe extern "C" fn current_el0_synchronous(e: &mut ExceptionContext);
// unsafe extern "C" fn current_el0_irq(e: &mut ExceptionContext);
// unsafe extern "C" fn current_el0_serror(e: &mut ExceptionContext);

// unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext);
// unsafe extern "C" fn current_elx_irq(e: &mut ExceptionContext);
// unsafe extern "C" fn current_elx_serror(e: &mut ExceptionContext);

// unsafe extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext);
// unsafe extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext);
// unsafe extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext);

// unsafe extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext);
// unsafe extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext);
// unsafe extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext);


#[no_mangle]
unsafe extern "C" fn current_el0_synchronous(e: &mut ExceptionContext) {
    let uart = Uart0::default();
    uart.puts("exceptions::current_el0_synchronous(): Caught exception.\r\n");
    e.elr_el1 += 4; //Return to first instruction after exception.
    uart.puts("exceptions::current_el0_synchronous(): Returning.\r\n");
}

#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    let uart = Uart0::default();
    uart.puts("exceptions::current_elx_synchronous(): Caught exception.\r\n");
    e.elr_el1 += 4; //Return to first instruction after exception.
    uart.puts("exceptions::current_elx_synchronous(): Returning.\r\n");
}

#[no_mangle]
unsafe extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
    let uart = Uart0::default();
    uart.puts("exceptions::current_elx_serror(): Caught exception.\r\n");
    e.elr_el1 += 4; //Return to first instruction after exception.
    uart.puts("exceptions::current_elx_serror(): Returning.\r\n");
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
    let uart = Uart0::default();
    uart.puts("exceptions::lower_aarch64_synchronous(): Caught exception.\r\n");
    e.elr_el1 += 4; //Return to first instruction after exception.
    uart.puts("exceptions::lower_aarch64_synchronous(): Returning.\r\n");
}

// unsafe extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext);
// unsafe extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext);

#[no_mangle]
unsafe extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
    let uart = Uart0::default();
    uart.puts("exceptions::lower_aarch32_synchronous(): Caught exception.\r\n");
    e.elr_el1 += 4; //Return to first instruction after exception.
    uart.puts("exceptions::lower_aarch32_synchronous(): Returning.\r\n");
}
