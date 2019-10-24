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

use crate::memory::{get_virt_addr_properties, AttributeFields};
use crate::uart;
use cortex_a::{barrier, regs::*};
use register::register_bitfields;

register_bitfields! {
    u64,
    
//AArch64 Reference Manual page 2150
    STAGE1_DESCRIPTOR [
///Privileged execute-never
/// Accesses to the block of addresses are prevented from being run as code.
///
/// When clear accesses to the block of addresses may be fetched and
/// executed as code.
        PXN OFFSET(53) NUMBITS(1) [],

///Various address fields, depending on use case
        LVL2_OUTPUT_ADDR_4KiB    OFFSET(21) NUMBITS(27) [], // [47:21]
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
            Block = 0,
            Table = 1
        ],

///Signals to MMU that the address block pointed at by descriptor is invalid.
        VALID OFFSET(0) NUMBITS(1) []
    ]
}

const FOUR_KIB: usize = 4 * 1024;
const FOUR_KIB_SHIFT: usize = 12; // log2(4 * 1024)

const TWO_MIB: usize = 2 * 1024 * 1024;
const TWO_MIB_SHIFT: usize = 21; // log2(2 * 1024 * 1024)

/// A descriptor pointing to the next page table.
struct TableDescriptor(register::FieldValue<u64, STAGE1_DESCRIPTOR::Register>);

impl TableDescriptor {
    fn new(next_lvl_table_addr: usize) -> Result<TableDescriptor, &'static str> {
        if next_lvl_table_addr % FOUR_KIB != 0 {
            return Err("TableDescriptor: Address is not 4 KiB aligned.");
        }

        let shifted = next_lvl_table_addr >> FOUR_KIB_SHIFT;

        Ok(TableDescriptor(
            STAGE1_DESCRIPTOR::VALID::True
                + STAGE1_DESCRIPTOR::TYPE::Table
                + STAGE1_DESCRIPTOR::NEXT_LVL_TABLE_ADDR_4KiB.val(shifted as u64),
        ))
    }

    fn value(&self) -> u64 {
        self.0.value
    }
}
 
