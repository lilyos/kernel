OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

ENTRY(_start)

SECTIONS {
    . = 0xffffffff80000000;
    .dynsym : { *(.dynsym) }
    .gnu.hash : { *(.gnu.hash) }
    .hash : { *(.hash) }
    .dynstr : { *(.dynstr) }
    .rela.dyn : { *(.rela.dyn) }
    . = ALIGN(0x1000);
    .rodata : { *(.rodata*) }
    . = ALIGN(0x1000);
    .eh_frame_hdr : { *(.eh_frame_hdr) }
    .eh_frame : { *(.eh_frame) }
    . = ALIGN(0x1000);
    .text : { *(.text*) }
    . = ALIGN(0x1000);
    .data.rel.ro : { *(.data.rel.ro*) }
    .dynamic : { *(.dynamic) }
    .got : { *(.got) }
    . = ALIGN(0x1000);
    .data : { *(.data*) }
    . = ALIGN(0x1000);
    .bss : { *(.bss*) }
    . = ALIGN(0x1000);
    .comment : { *(.comment) }
    .symtab : { *(.symtab) }
    .shstrtab : { *(.shstrtab) }
    .strtab : { *(.strtab) }
    .stivale2hdr : { KEEP(*(.stivale2hdr)) } : .rodata
}