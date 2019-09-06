#![no_std]
#![no_main]

use core::panic::PanicInfo;
//use peripherals::{i2c, mbox, uart};
use peripherals::PERIPHERALS;
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
/// Main loop processes inputs and delivers outputs.
///
#[export_name = "main"] //So startup.rs can find fn main().
fn main() -> ! {
    let mut ultra2 = ultra2::Ultra2::new();

    PERIPHERALS.init();

    if let Err(err) = ultra2.init() {
        PERIPHERALS.uart.puts("main(): Error ultra2.init() failed - ");        
        PERIPHERALS.uart.puts(err.msg());
    } else {
        PERIPHERALS.uart.puts("UART Initialized.\r\n");
    }

    loop {
//Hello world.
    }
}
