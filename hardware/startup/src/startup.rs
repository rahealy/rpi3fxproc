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

/************************** Register ******************************/

// use register::{cpu::RegisterReadWrite, register_bitfields};
// 
// register_bitfields! {
//     u32,
//     CPACR_EL1 [
//         ///Not used.
//         TTA OFFSET(28) NUMBITS(1) [],
// 
//         ///Traps instructions that access registers associated with floating-point
//         ///and SIMD execution to trap to EL1 when executed from EL0 or EL1
//         ///
//         /// 0b00 Trap any instruction in EL0 or EL1
//         /// 0b01 Trap any instruction in EL0 
//         /// 0b10 Trap any instruction in EL0 or EL1
//         /// 0b11 No instructions are trapped
//         FPEN OFFSET(20) NUMBITS(2) []
//     ]
// }
// 
// pub struct Reg;
// 
// impl RegisterReadWrite<u32, CPACR_EL1::Register> for Reg {
//     #[inline]
//     fn get(&self) -> u32 {
//         let reg;
//         unsafe {
//             asm!("mrs $0, CPACR_EL1" : "=r"(reg) ::: "volatile");
//         }
//         reg
//     }
// 
//     #[inline]
//     fn set(&self, value: u32) {
//         unsafe {
//             asm!("msr CPACR_EL1, $0" :: "r"(value) :: "volatile")
//         }
//     }
// }
// 
// impl Reg {}
// 
// pub static CPACR_EL1: Reg = Reg {};


/************************** Startup Code ******************************/

///
/// First unsafe rust code. Initalize RPi MMU, set up exception
/// handlers and transition to safe rust code in main().
///
#[export_name = "unsafe_main"]
pub unsafe fn __unsafe_main() -> ! {
    use crate::uart;
    use crate::memory;
    use crate::exceptions;

    extern "Rust" {
        fn main() -> !; //Forward declaration of main().
    }

//Initialze startup subsystems.
    uart::Uart0::init();
    memory::init();

    uart::Uart0::default().puts("startup::__unsafe_main(): Calling main().\r\n");
    exceptions::init();
    main();
}


///
/// Rust init. Zero the bss segment and transition to rust code.
///
#[no_mangle]
pub unsafe extern "C" fn rinit() -> ! {
    extern "C" { //Provided by linker
        static mut __bss_beg: u64;
        static mut __bss_end: u64;
    }
    r0::zero_bss(&mut __bss_beg, &mut __bss_end);
    extern "Rust" {
        fn unsafe_main() -> !; //Forward declaration of unsafe_main().
    }
    unsafe_main(); //Transition from unsafe 'C' to unsafe rust.
}



///
/// Run first. Initializes the RPi CPU and cores and drops into 
/// Execution state 1 (EL1/Operating System).
///
#[link_section = ".text.boot"]
#[no_mangle]
pub unsafe extern "C" fn _boot() -> ! {
    use cortex_a::{asm, barrier, regs::*};
    use super::STACK_START;

    const CORE_0:      u64 = 0;
    const CORE_MASK:   u64 = 0x3;
    const EL2:         u32 = CurrentEL::EL::EL2.value;

    if CORE_0 == MPIDR_EL1.get() & CORE_MASK && EL2 == CurrentEL.get() {
        if EL2 == CurrentEL.get() { //Need to change to EL1

//Set up access to timers.
            CNTHCTL_EL2.write(
                CNTHCTL_EL2::EL1PCEN::SET  + //Allow access to the physical timer registers.
                CNTHCTL_EL2::EL1PCTEN::SET   //Allow access to the physical counter registers.
            );

            CNTVOFF_EL2.set(0); //Virtual timer same as physical timer (0 offset.)


//For description of CPACR_EL1 See section 4.3.32 in the
// ARM® Cortex®-A57 MPCore Processor
// Revision: r1p3
// Technical Reference Manual
            //Don't trap the SIMD and FP operations in EL[0,1]
            asm!("msr CPACR_EL1, $0" :: "r"(0x300000) :: "volatile");
            //Default 0x33ff is fine.
            asm!("msr CPTR_EL2, $0" :: "r"(0x33ff) :: "volatile");
            //Don't trap anything in the hypervisor.
            asm!("msr HSTR_EL2, xzr" :::: "volatile");
            barrier::isb(barrier::SY);

//Set up architecture.
            HCR_EL2.modify (
//                 HCR_EL2::IMO::CLEAR +
//                 HCR_EL2::FMO::CLEAR +                
                HCR_EL2::RW::EL1IsAarch64
            );
 
//Set up for transition to EL1. At this point whatever is in the SPSR_EL2 
//register is undefined. Mask off bits so the PSTATE register isn't set
//to whatever garbage is in SPSR_EL2 when we make the transition.
            SPSR_EL2.write (
                SPSR_EL2::D::Masked + //Whatever here isn't returned.
                SPSR_EL2::A::Masked + //Whatever here isn't returned.
                SPSR_EL2::I::Masked + //Whatever here isn't returned.
                SPSR_EL2::F::Masked + //Whatever here isn't returned.
                SPSR_EL2::M::EL1h     //On eret return to EL1.
            );

//Set address of function to jump to after transition to EL1.
            ELR_EL2.set(rinit as *const () as u64); //eret jumps to rinit()

            SP_EL1.set(STACK_START);
            asm::eret();
        } else {
            SP.set(STACK_START);
            rinit()
        }
    }

// if not core0, infinitely wait for events
    loop {
        asm::wfe();
    }
}
