#![no_std]
#![no_main]

use core::panic::PanicInfo;

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
    loop {
//Hello world.
    }
}
