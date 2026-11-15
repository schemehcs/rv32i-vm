ENTRY(_start)
MEMORY {
    /* Adjust ORIGIN and LENGTH to match your emulator's RAM */
    RAM (rwx) : ORIGIN = 0x00000000, LENGTH = 64K
}
SECTIONS {
    .text : {
        *(.text.init)
        *(.text*)
    } > RAM
    .rodata : { *(.rodata*) } > RAM
    .data : { *(.data*) } > RAM
    .bss : { *(.bss*) *(COMMON) } > RAM
    _stack_top = ORIGIN(RAM) + LENGTH(RAM);
}
