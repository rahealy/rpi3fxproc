/*
 * MIT License
 *
 * Parts Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
 * Parts Copyright (c) 2019 Richard Healy
 *
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

/*
 * Set up virtual memory.
 */

use cortex_a::{barrier, regs::*};
use super::MMIO_BASE;
use register::register_bitfields;
use core::ops;
use crate::uart::Uart0;

register_bitfields! {
    u64,
    
///
///DESCRIPTOR
/// A single entry in a table 
    
    DESCRIPTOR [
///Privileged execute-never
/// Accesses to the block of addresses are prevented from being run as code.
///
/// When clear accesses to the block of addresses may be fetched and
/// executed as code.
        PXN OFFSET(53) NUMBITS(1) [],

///Various address fields, depending on use case
        LVL2_OUTPUT_ADDR_4KiB OFFSET(21) NUMBITS(27) [], // [47:21]
        LVL3_OUTPUT_ADDR_4KiB OFFSET(12) NUMBITS(36) [], // [47:12]
        NEXT_LVL_TABLE_ADDR_4KiB OFFSET(12) NUMBITS(36) [], // [47:12]

///Access Flag
/// When set the MMU grants access to the address block.
///
/// When clear the MMU triggers an access flag fault resulting in a
/// synchronous exception which is handled by software. Reasons are
/// stored in ESR_EL[1,2,3].
        AF OFFSET(10) NUMBITS(1) [],

///Shareability field. FIXME: Not sure what this means for this hardware.
        SH OFFSET(8) NUMBITS(2) [
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],

///Permissions determine whether and how the address block can be 
///accessed by a given CPU execution level and the lesser 
///privileged execution levels beneath it.
        AP OFFSET(6) NUMBITS(2) [
            RW_EL1     = 0b00,
            RW_EL1_EL0 = 0b01,
            RO_EL1     = 0b10,
            RO_EL1_EL0 = 0b11
        ],

///Tell MMU to use one of the memory attributes (0..7) set in the 
///MAIR_EL1 register for all accesses to the address block pointed
///at by this descriptor.
        AttrIndx OFFSET(2) NUMBITS(3) [],

///Indicates whether this descriptor points at an address block or
///the beginning of the next level's table.
        TYPE OFFSET(1) NUMBITS(1) [
            BLOCK = 0,
            TABLE = 1
        ],

///Signals to MMU that the address block pointed at by descriptor is invalid.
        VALID OFFSET(0) NUMBITS(1) []
    ]
}


///MMU expects page table to be 4096 bytes divided into 512 64bit entries
///aligned on a 4096 byte boundary.
#[repr(C)]
#[repr(align(4096))]
struct PageTable ([u64; 512]);

impl ops::Deref for PageTable {
    type Target = [u64; 512];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for PageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PageTable {
    fn ptr(&self) -> *const PageTable {
        self as *const _
    }

    const fn new() -> PageTable {
        PageTable([0; 512])
    }
}

unsafe fn ro_end() -> usize {
    extern "C" {
        static __ro_end: u64;
    }
    return __ro_end as usize;
}


/// The LVL2 page table containng the 2 MiB entries.
static mut LVL2_TABLE: PageTable = PageTable::new();

/// The LVL3 page table containing the 4 KiB entries.
static mut LVL3_TABLE: PageTable = PageTable::new();

pub unsafe fn init() {
    let uart = Uart0::default();

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
//Attribute 1 - General Memory
//These tell the MMU that the address blocks with attribute 1 are mapped
//to memory like DRAM.
        MAIR_EL1::Attr1_HIGH::Memory_OuterWriteBack_NonTransient_ReadAlloc_WriteAlloc +
        MAIR_EL1::Attr1_LOW_MEMORY::InnerWriteBack_NonTransient_ReadAlloc_WriteAlloc  +
//Attribute 0 - Device
//These tell the MMU that address blocks with attribute 0 are mapped
//to devices like the MMIO peripherals.
        MAIR_EL1::Attr0_HIGH::Device +
        MAIR_EL1::Attr0_LOW_DEVICE::Device_nGnRE
    );

//Helper function.
    let print_range = |beg: usize, end: usize, div: u64| {
        uart.u64hex(beg as u64);
        uart.puts("-");
        uart.u64hex(end as u64);
        uart.puts(" [");
        uart.u64hex((beg as u64) / div);
        uart.puts("-");
        uart.u64hex((end as u64) / div);
        uart.puts("]");
    };

///////////////////////////////////////////////////////////////////////
//The level 2 table covers the first 1GiB of the address space. Each
//entry in the level 2 table describes 2MiB (0x20_0000) in the virtual
//address space.
///////////////////////////////////////////////////////////////////////

//The first 2MiB is subdivided into 512 4KiB (0x1000)   described
//by the 512 entries in the level 3 table. Tell the MMU to jump to the
//level 3 table when this block of addresses is accessed.
    uart.puts("mmu::init_page_tables(): Setting up level 2 table\r\n");

    print_range(0, 0x20_0000, 0x20_0000);
    uart.puts("->level 3 table.\r\n");
    LVL2_TABLE[0] = (DESCRIPTOR::VALID::SET  +
                     DESCRIPTOR::TYPE::TABLE +
                     DESCRIPTOR::NEXT_LVL_TABLE_ADDR_4KiB.val (
                        (LVL3_TABLE.ptr() as u64) >> 12
                    )).value;


//Entries describe DRAM up to but not including the peripheral addresses
//starting at MMIO_BASE.
    print_range(0x20_0000, MMIO_BASE as usize, 0x20_0000);
    uart.puts("\r\n");

    for (i,desc) in LVL2_TABLE.iter_mut()
                              .take((MMIO_BASE / 0x20_0000) as usize)
                              .enumerate()
                              .skip(1) 
    {
        *desc = (DESCRIPTOR::TYPE::BLOCK        + //Block.
                 DESCRIPTOR::VALID::SET         + //Block is valid.
                 DESCRIPTOR::AF::SET            + //Block is accessible.
                 DESCRIPTOR::SH::InnerShareable + //Block is inner sharable.
                 DESCRIPTOR::AttrIndx.val(1)    + //Block is DRAM - see MAIR_EL1 above.
                 DESCRIPTOR::AP::RW_EL1         + //Block is Read/Writable.
                 DESCRIPTOR::PXN::SET           + //Block can never execute code. 
                 DESCRIPTOR::LVL2_OUTPUT_ADDR_4KiB.val(i as u64)).value;
    }

//Entries describe peripheral addresses starting at MMIO_BASE and exetending
//to the end of the 1GB address space covered by the LVL2_TABLE.
    print_range(MMIO_BASE as usize, 0x4000_0000, 0x20_0000);
    uart.puts("\r\n");

    for (i,desc) in LVL2_TABLE.iter_mut()
                              .enumerate()
                              .skip((MMIO_BASE / 0x20_0000) as usize) //Start at MMIO_BASE
    {
        *desc = (DESCRIPTOR::TYPE::BLOCK        + //Block.
                 DESCRIPTOR::VALID::SET         + //Block is valid.
                 DESCRIPTOR::AF::SET            + //Block is accessible.
                 DESCRIPTOR::SH::OuterShareable + //Block is outer sharable.
                 DESCRIPTOR::AttrIndx.val(0)    + //Block is a device - see MAIR_EL1 above.
                 DESCRIPTOR::AP::RW_EL1         + //Block is Read/Writable.
                 DESCRIPTOR::PXN::SET           + //Block can never execute code.
                 DESCRIPTOR::LVL2_OUTPUT_ADDR_4KiB.val(i as u64)).value;
    }

///////////////////////////////////////////////////////////////////////
//The level 3 table covers the first 2MiB of address space. Each entry
//in the level 3 table describes 4KiB (0x1000) in the virtual address 
//space. 
///////////////////////////////////////////////////////////////////////

//Entries describe DRAM up to but not including the read only 
//executable code.
    uart.puts("mmu::init_page_tables(): Setting up level 3 table\r\n");
    print_range(0, 0x80000, 0x1000);
    uart.puts("\r\n");

    for (i,desc) in LVL3_TABLE.iter_mut()
                              .take(0x80000 / 0x1000)
                              .enumerate()
    {
        *desc = (DESCRIPTOR::TYPE::TABLE        + //Table
                 DESCRIPTOR::VALID::SET         + //Block is valid.
                 DESCRIPTOR::AF::SET            + //Block is accessible.
                 DESCRIPTOR::SH::InnerShareable + //Block is inner sharable.
                 DESCRIPTOR::AttrIndx.val(1)    + //Block is DRAM - see MAIR_EL1 above.
                 DESCRIPTOR::AP::RW_EL1         + //Block is Read/Writable.
                 DESCRIPTOR::PXN::SET           + //Block can never execute code.
                 DESCRIPTOR::LVL3_OUTPUT_ADDR_4KiB.val(i as u64)).value;
    }

//Entries describe DRAM containing read only code and data.      
    print_range(0x80000, ro_end(), 0x1000);
    uart.puts("\r\n");

    for (i,desc) in LVL3_TABLE.iter_mut()
                              .take(ro_end() / 0x1000) //Range includes read only blocks. 
                              .enumerate()
                              .skip(0x80000 / 0x1000) //Skip to the first read only block.
    {
        *desc = (DESCRIPTOR::TYPE::TABLE        + //Table
                 DESCRIPTOR::VALID::SET         + //Block is valid.
                 DESCRIPTOR::AF::SET            + //Block is accessible.
                 DESCRIPTOR::SH::InnerShareable + //Block is inner sharable.
                 DESCRIPTOR::AttrIndx.val(1)    + //Block is DRAM - see MAIR_EL1 above.
                 DESCRIPTOR::AP::RO_EL1         + //Block is Read Only.
                 DESCRIPTOR::PXN::CLEAR         + //Block can execute code.
                 DESCRIPTOR::LVL3_OUTPUT_ADDR_4KiB.val(i as u64)).value;
    }

//Entries describe remaining DRAM in the 2MiB range.
    print_range(ro_end(), 0x20_0000, 0x1000);
    uart.puts("\r\n");

    for (i,desc) in LVL3_TABLE.iter_mut()
                              .enumerate()
                              .skip(ro_end() / 0x1000)  //Skip past the last read only block.
    {
        *desc = (DESCRIPTOR::TYPE::TABLE        + //Table
                 DESCRIPTOR::VALID::SET         + //Block is valid.
                 DESCRIPTOR::AF::SET            + //Block is accessible.
                 DESCRIPTOR::SH::InnerShareable + //Block is inner sharable.
                 DESCRIPTOR::AttrIndx.val(1)    + //Block is DRAM - see MAIR_EL1 above.
                 DESCRIPTOR::AP::RW_EL1         + //Block is Read/Writable.
                 DESCRIPTOR::PXN::SET           + //Block can never execute code.
                 DESCRIPTOR::LVL3_OUTPUT_ADDR_4KiB.val(i as u64)).value;
    }

//Point to the LVL2 table base address in TTBR0.
    uart.puts("mmu::init_page_tables(): Setting TTBR0_EL1 to table address.\r\n");
    TTBR0_EL1.set_baddr(LVL2_TABLE.ptr() as u64);
    uart.puts("mmu::init_page_tables(): Page tables initialized.\r\n");

//TCR_EL1
//Instructs the MMU on how the table is configured and how to cache the memory
//it points at.
    uart.puts("mmu::init(): Setting TCR_EL1.\r\n");
    let ips = ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange);
    TCR_EL1.write (
        TCR_EL1::TBI0::Ignored          + //Top Byte ignored for addresses in the TBR0_EL1 table.
        TCR_EL1::IPS.val(ips)           + //Intermediate Physical Address Size.
        TCR_EL1::TG0::KiB_4             + //Table granule (size in bytes) is 4KiB. 
        TCR_EL1::SH0::Inner             + //Inner sharable.
        TCR_EL1::EPD0::EnableTTBR0Walks + //Allow translation table walks.
        TCR_EL1::T0SZ.val(34)           + //Memory size is 2(64-T0SZ) bytes or 1GiB (0x4000_0000)
        TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable + //Outer memory cached.
        TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable   //Inner memory cached.
    );

//Enable MMU and caches.
    uart.puts("mmu::init(): Enabling MMU.\r\n");
    barrier::isb(barrier::SY);
    SCTLR_EL1.modify (
        SCTLR_EL1::M::Enable    + //Enable MMU for EL1.
        SCTLR_EL1::C::Cacheable + //Data access cacheable
        SCTLR_EL1::I::Cacheable   //Instruction access cacheable
    );
    barrier::isb(barrier::SY);
    uart.puts("mmu::init(): MMU Enabled.\r\n");
}
