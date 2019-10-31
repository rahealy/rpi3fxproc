#![no_std]
pub const MMIO_BASE: usize = 0x3F00_0000;         //Peripheral access is derived from this base memory offset.
pub mod startup;

