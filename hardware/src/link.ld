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

    .vectors ALIGN(2048):
    {
        *(.vectors)
    }

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

PROVIDE(current_el0_synchronous   = default_exception_handler);
PROVIDE(current_el0_irq           = default_exception_handler);
PROVIDE(current_el0_serror        = default_exception_handler);

PROVIDE(current_elx_synchronous   = default_exception_handler);
PROVIDE(current_elx_irq           = default_exception_handler);
PROVIDE(current_elx_serror        = default_exception_handler);

PROVIDE(lower_aarch64_synchronous = default_exception_handler);
PROVIDE(lower_aarch64_irq         = default_exception_handler);
PROVIDE(lower_aarch64_serror      = default_exception_handler);

PROVIDE(lower_aarch32_synchronous = default_exception_handler);
PROVIDE(lower_aarch32_irq         = default_exception_handler);
PROVIDE(lower_aarch32_serror      = default_exception_handler);
