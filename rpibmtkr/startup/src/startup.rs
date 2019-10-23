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


///
/// Run first. Initializes the RPi CPU and cores and drops into 
/// Execution state 1 (EL1/Operating System).
///
#[link_section = ".text.boot"]
#[no_mangle]
pub unsafe extern "C" fn _boot() -> ! {
    use cortex_a::{asm, regs::*};

    const CORE_0:      u64 = 0;
    const CORE_MASK:   u64 = 0x3;
    const STACK_START: u64 = 0x80_000;
    const EL2:         u32 = CurrentEL::EL::EL2.value;

    if CORE_0 == MPIDR_EL1.get() & CORE_MASK && EL2 == CurrentEL.get() {
        if EL2 == CurrentEL.get() { //Need to change to EL1

//Set up access to timers.
            CNTHCTL_EL2.write(
                CNTHCTL_EL2::EL1PCEN::SET  + //Allow access to the physical timer registers.
                CNTHCTL_EL2::EL1PCTEN::SET   //Allow access to the physical counter registers.
            );

            CNTVOFF_EL2.set(0); //Virtual timer same as physical timer (0 offset.)

//Set up architecture.
            HCR_EL2.modify(HCR_EL2::RW::EL1IsAarch64);
 
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

//
// MAIR_EL1
//
// Describe to the MMU the attributes the memory/devices will have when they get 
// mapped into the virtual address space. MAIR_EL1 can have up to 8 different
// attributes. We just need two - one for DRAM and one for the MMIO peripherals.
//
// A field in a page table entry stored in DRAM references one of the attributes
// set here. When the MMU reads a page table entry it uses the attribute (0..7)
// for that memory access.
//
// High 4 bits sets cache hints applying to accesses from regions of the CPU
// considered to be 'outer' relative to the memory/device being accessed.
//
// Low 4 bits sets cache hints applying to accesses from regions of the CPU
// considered to be 'inner' relative to the memory/ device being accessed.
//
            MAIR_EL1.write (
//
//Attribute 1 - General Memory
//These tell the MMU that the address blocks with attribute 1 are mapped
//to memory like DRAM.
//
                MAIR_EL1::Attr1_HIGH::Memory_OuterWriteBack_NonTransient_ReadAlloc_WriteAlloc +
                MAIR_EL1::Attr1_LOW_MEMORY::InnerWriteBack_NonTransient_ReadAlloc_WriteAlloc  +
//
//Attribute 0 - Device
//These tell the MMU that address blocks with attribute 0 are mapped
//to the MMIO peripherals.
//
                MAIR_EL1::Attr0_HIGH::Device +
                MAIR_EL1::Attr0_LOW_DEVICE::Device_nGnRE
            );

//Enable MMU and caches.
            SCTLR_EL1.modify (
                SCTLR_EL1::M::Enable    + //Enable MMU for EL1.
                SCTLR_EL1::C::Cacheable + //Data access cacheable
                SCTLR_EL1::I::Cacheable   //Instruction access cacheable
            );

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
