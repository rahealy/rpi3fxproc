/*
 * link.ld
 *  Linker script.
 */

ENTRY(_setup_rpi3) /* In setup.S */

SECTIONS
{
/* RPi3 starts executing code at 0x80000 */    
/*    . = 0x80000;*/ 

/* The RPi will load bootloader code from the SD card to 0x80000 
 * regardless of what we specify in the linker script. setup.S will move
 * the bootloader code to 0x80000 - 2048 so the bootloader isn't 
 * overwritten by the code it's trying to load into 0x80000. Because of
 * this we need to link our code so it expects to be at 0x80000 - 2048
 * rather than 0x80000.
 */
    . = 0x80000 - 2048;

/* First memory location used by the code.*/
    _code_begin = .;
    
/* Keep text segments including the .text.setup section in setup.S */
    .text :
    {
        KEEP(*(.text.setup)) *(.text .text.*) 
    }

/* Read only program data */
    .rodata :
    {
        *(.rodata .rodata.*)
    }

/* Data that could be changed by program */
    .data :
    {
        *(.data .data.*)
    }

/* Uninitialzed reserved program data. This gets zeroed out by rinit() in main.rs. */
    .bss ALIGN(8):
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

/* First unused memory location. */
    _code_end = .;
    
/* These segments may be generated during compilation and aren't necessary */
    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) } 
}

/* Calculate the size of the code for setup.S */
__code_sz = (_code_end - _code_begin) >> 3;
