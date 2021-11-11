ENTRY(_start)
 
SECTIONS
{
    . = 0x8000;
    __start = .;
    __text_start = .;
    .text :
    {
        KEEP(*(.text.boot))
        *(.text)
    }
    . = ALIGN(4096);
    __text_end = .;
 
    __rodata_start = .;
    .rodata :
    {
        *(.rodata)
    }
    . = ALIGN(4096);
    __rodata_end = .;
 
    __data_start = .;
    .data :
    {
        *(.data)
    }
    . = ALIGN(4096);
    __data_end = .;
 
    __bss_start = .;
    .bss :
    {
        bss = .;
        *(.bss)
    }
    . = ALIGN(16);
	__stack_end__ = .;
	. += 0x10000;
	__stack_top_core3__ = .;
	. += 0x10000;
	__stack_top_core2__ = .;
	. += 0x10000;
	__stack_top_core1__ = .;
	. += 0x01000; /* stack space EL2 mode 4kB - stack goes backwards from memory end towards begin*/
	__stack_top_EL2__ = .;
	. += 0x0E000;
	__stack_top_EL1__ = .;
	. += 0x01000;
	__stack_top_EL0__ = .;
	__stack_top_core0__ = .;
	__stack_top__ = .;
	/* the heap memory address space starts where the executable and the static variables ends (aligned to 4kB to fit into MMU page)*/
    . = ALIGN(4096);
	__heap_start = .;
    /* heap end is defined by the usage split of arm/gpu - set this to a fixed value for the time beeing */
	__heap_end = 0x3E000000;
}