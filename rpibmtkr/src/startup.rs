#![deny(missing_docs)]
#![deny(warnings)]

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


#[link_section = ".text.boot"]
#[no_mangle]
pub unsafe extern "C" fn _boot() -> ! {
    use cortex_a::{asm, regs::*};

    const CORE_0:      u64 = 0;
    const CORE_MASK:   u64 = 0x3;
    const STACK_START: u64 = 0x80_000;
    const EL2:         u32 = CurrentEL::EL::EL2.value;

    if (CORE_0 == MPIDR_EL1.get() & CORE_MASK) &&
       (EL2 == CurrentEL.get())
    {
        SP.set(STACK_START);
        rinit()
    } else {
        // if not core0, infinitely wait for events
        loop {
            asm::wfe();
        }
    }
}
