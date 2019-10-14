/*
 * MIT License
 *
 * Parts Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
 * Parts Copyright (c) 2019 Richard Healy
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


/************************** Startup Code ******************************/

///
/// Transition from unsafe rust code to safe rust code in main() 
/// function.
///
#[export_name = "unsafe_main"]
pub unsafe fn __unsafe_main() -> ! {
    extern "Rust" {
        fn main() -> !; //Forward declaration of main().
    }
    main();
}


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
    extern "Rust" {
        fn unsafe_main() -> !; //Forward declaration of unsafe_main().
    }
    unsafe_main(); //Transition from unsafe 'C' to unsafe rust.
}

//
// Assembler in setup.S initializes the RPi hardware then jumps to the
// unsafe 'C' style rinit() function above.
//
global_asm!(include_str!("setup.S"));
