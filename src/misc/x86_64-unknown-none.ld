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
    . += CONSTANT(MAXPAGESIZE);
    .rodata : { *(.rodata*) }
    . += CONSTANT(MAXPAGESIZE);
    .eh_frame_hdr : { *(.eh_frame_hdr) }
    .eh_frame : { *(.eh_frame) }
    . += CONSTANT(MAXPAGESIZE);
    .text : { *(.text*) }
    . += CONSTANT(MAXPAGESIZE);
    .data.rel.ro : { *(.data.rel.ro*) }
    .dynamic : { *(.dynamic) }
    .got : { *(.got) }
    . += CONSTANT(MAXPAGESIZE);
    .data : { *(.data*) }
    . += CONSTANT(MAXPAGESIZE);
    .bss : { *(.bss*) }
    . += CONSTANT(MAXPAGESIZE);
    .comment : { *(.comment) }
    .symtab : { *(.symtab) }
    .shstrtab : { *(.shstrtab) }
    .strtab : { *(.strtab) }
    .stivale2hdr : { KEEP(*(.stivale2hdr)) } : .rodata
}