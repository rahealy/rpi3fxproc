/*
 * link.ld
 *  Linker script.
 *    
    .vectors ALIGN(2048) :
    {
        *(.vectors)
    }
 */

ENTRY(_boot);

SECTIONS
{
    . = 0x80000; /* This is already 4KiB aligned */
    .text :
    {
        KEEP(*(.text.boot)) *(.text .text.*)
    }

    . = ALIGN(4096); /* Fill up to 4KiB */

    .rodata :
    {
        *(.rodata .rodata.*)
    }
    . = ALIGN(4096); /* Fill up to 4KiB */
    __ro_end = .;
 
    .data :
    {
        *(.data .data.*)
    }

    .bss ALIGN(8):
    {
        __bss_beg = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
