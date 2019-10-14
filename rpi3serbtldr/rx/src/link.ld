/*
 * link.ld
 *  Linker script.
 */

ENTRY(_boot);

SECTIONS
{
/* The RPi will load bootloader code from the SD card to 0x80000 
 * regardless of what we specify in the linker script. setup.S will move
 * the bootloader code to 0x80000 - 4096 so the bootloader isn't 
 * overwritten by the code it's trying to load into 0x80000. Because of
 * this we need to link our code so it expects to be at 0x80000 - 4096
 * rather than 0x80000.
 */
    . = 0x80000 - 4096;

    /* First memory location used by the code.*/
    __code_begin = .;

    .text :
    {
        KEEP(*(.text.boot)) *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss ALIGN(8):
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

/* First unused memory location. */
    __code_end = .;

    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}

/* Calculate the size of the code for setup.S */
__code_sz = (__code_end - __code_begin) >> 3;

