#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]

use core::panic::PanicInfo;
use peripherals::uart::Uart0;
use peripherals::gpfsel::GPFSEL;

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
/// Send a break signal to tx
///
/// #Arguments
///
/// * `uart` - initialized uart
///
#[inline]
fn send_break_signal(uart: &Uart0) -> () {
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
#[inline]
fn get_le_u32(uart: &Uart0) -> (u32) {
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
#[inline]
fn send_ok(uart: &Uart0) -> () {
    uart.send('O');
    uart.send('K');
}


///
/// Send ER (Error).
///
/// #Arguments
///
/// * `uart` - initialized uart
///
#[inline]
fn send_error(uart: &Uart0) -> () {
    uart.send('E');
    uart.send('R');
}


enum State {
    POLL,
    JTAG,
    LOAD,
    JUMP,
    WAIT
}

///
/// Receive code over serial port UART0 and execute.
///
#[export_name = "main"] //So startup.rs can find fn main().
fn main() -> ! {    
    Uart0::init();

    let uart = Uart0::default();
    let gpfsel = GPFSEL::default();
    let lodptr: *mut u8 = 0x80000 as *mut u8;
    let mut state = State::POLL;

//Hello world.
    uart.puts("rpiserbtldr_rx\r\n");
    for _ in 0..50 { uart.send('.'); }
    uart.puts("\r\n");

//Let tx know we're ready.
    send_break_signal(&uart);

//State machine.
    loop {
        match state {
            State::POLL => { //Wait for instructions.
                match get_le_u32(&uart) {
                    0x4A544147 => { //'J','T','A','G'
                        state = State::JTAG;
                    },

                    0x4C4F4144 => { //'L','O','A','D'
                        state = State::LOAD;
                    },

                    0x4A554D50 => { //'J','U','M','P'
                        state = State::JUMP;
                    },

                    0x57414954 => { //'W','A','I','T'
                        state = State::WAIT;
                    },

                    _ => {
                        send_error(&uart);
                    }
                }
            },

            State::JTAG => { //Enable JTAG Pins.
                gpfsel.fsel_jtag();
                state = State::POLL;
                send_ok(&uart);
            },

            State::LOAD => { //Load code and execute.
//Get pending data size in bytes from tx.
                let sz = get_le_u32(&uart);
                if sz > 500000000 { //500MB seems okay, no? 
                    send_error(&uart);
                } else {
//Load tx'd data into memory.
                    send_ok(&uart);
                    unsafe {
                        for i in 0..sz {
                            *lodptr.offset(i as isize) = uart.getu8();
                        }
                    }
                    send_ok(&uart);
                }

                state = State::POLL;
            },

            State::JUMP => {
//Jump to loaded code. Byeee!
                let jmplod: extern "C" fn() -> ! = unsafe {
                    core::mem::transmute(lodptr as *const ())
                };
                jmplod();
            },

            State::WAIT => { //Wait in an infinite loop.
                send_ok(&uart);
                loop {}
            }
        }
    }
}
