OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

ENTRY(_start)

SECTIONS {
    . = 0xffffffff80000000;
    PROVIDE(__KERNEL_START = .);
    
    . = ALIGN(0x1000);
    .PROVIDE(__RODATA_START = .);
    .rodata : { *(.rodata*) }
    .PROVIDE(__RODATA_END = .);

    . = ALIGN(0x1000);
    PROVIDE(__TEXT_START = .);
    .text : { *(.text*) }
    PROVIDE(__TEXT_END = .);

    . = ALIGN(0x1000);
    PROVIDE(__DATA_START = .);
    .data : { *(.data*) }
    PROVIDE(__DATA_END = .);

    . = ALIGN(0x1000);
    PROVIDE(__BSS_START = .);
    .bss : { *(.bss*) }
    PROVIDE(__BSS_END = .);

    . = ALIGN(0x1000);
    PROVIDE(__MISC_START = .);
    .data.rel.ro : { *(.data.rel.ro*) }
    .rela.dyn : { *(.rela.dyn) }
    .got : { *(.got) }
    .symtab : { *(.symtab) }
    .shstrtab : { *(.shstrtab) }
    .strtab : { *(.strtab) }
    PROVIDE(__MISC_END = .);
    PROVIDE(__KERNEL_END = .);
}