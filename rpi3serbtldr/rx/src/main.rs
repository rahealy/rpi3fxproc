#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]


/************************** Startup Code ******************************/

///
/// Transition from unsafe rust code to safe rust code in main() 
/// function.
///
#[export_name = "unsafe_main"]
pub unsafe fn __unsafe_main() -> ! { main(); }


///
/// Rust init. Zero the bss segment and transition to rust code.
///
#[no_mangle]
pub unsafe extern "C" fn rinit() -> ! {
    extern "C" { //Provided by linker
        static mut __bss_start: u64;
        static mut __bss_end: u64;
    }

    r0::zero_bss(&mut __bss_start, &mut __bss_end);

//Transition from unsafe 'C' to unsafe rust.
    extern "Rust" { fn unsafe_main() -> !; }
    unsafe_main();
}

 
//
// Assembler in setup.S initializes the RPi hardware then jumps to the
// unsafe 'C' style rinit() function.
//
global_asm!(include_str!("setup.S"));


/************************** Main Code *********************************/

use core::panic::PanicInfo;

const MMIO_BASE: u32 = 0x3F00_0000; //Used by gpio

mod gpio;
mod mbox;
mod uart;

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
/// Send a break signal to tx
///
/// #Arguments
///
/// * `uart` - initialized uart
///
fn send_break_signal(uart: &uart::Uart) -> () {
    for _ in 0..3 {
        uart.send(0x03 as char);
    }
}


///
/// Read 4 bytes and interpret it as a little endian unsigned int.
///
/// #Arguments
///
/// * `uart` - initialized uart
///
fn get_le_u32(uart: &uart::Uart) -> (u32) {
    let mut fsize: u32 = u32::from(uart.getc());
    fsize |= u32::from(uart.getc()) << 8;
    fsize |= u32::from(uart.getc()) << 16;
    fsize |= u32::from(uart.getc()) << 24;
    return fsize;
}


///
/// Send OK.
///
/// #Arguments
///
/// * `uart` - initialized uart
///
fn send_ok(uart: &uart::Uart) -> () {
    uart.send('O');
    uart.send('K');
}


///
/// Send SE (Size Exceeded).
///
/// #Arguments
///
/// * `uart` - initialized uart
///
fn send_size_exceeded(uart: &uart::Uart) -> () {
    uart.send('S');
    uart.send('E');
}


///
/// Receive code over serial port UART0 and execute.
///
fn main() -> ! {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();

    if uart.init(&mut mbox).is_err() {
        panic!();
    }

//Hello world.
    for c in "rpiserbtldr\r\n".chars() { 
        uart.send(c); 
    }

//Let tx know we're ready for data.
    send_break_signal(&uart);

//Get pending data size in bytes from tx.
    let sz = get_le_u32(&uart);
    if sz < 500000000 { //500MB seems okay, no? 
        send_ok(&uart);
    } else {
        send_size_exceeded(&uart);
        panic!();
    }

//Load tx'd data into memory starting at 0x80000
    let lodptr: *mut u8 = 0x80000 as *mut u8;
    unsafe {
        for i in 0..sz {
            *lodptr.offset(i as isize) = uart.getc();
        }
    }

    send_ok(&uart);

//Jump to loaded code.
    let jmplod: extern "C" fn() -> ! = unsafe { 
        core::mem::transmute(lodptr as *const ()) 
    };

    jmplod();
}
