/* Simple linker script for the JOS kernel.
   See the GNU ld 'info' manual ("info ld") to learn the syntax. */

OUTPUT_FORMAT("elf32-i386", "elf32-i386", "elf32-i386")
OUTPUT_ARCH(i386)
ENTRY(_start)

SECTIONS
{
	/* Link the kernel at this address: "." means the current address */
        /* Must be equal to KERNLINK */
	. = 0x80100000;
	LOAD_OFFSET = 0x80000000;

	.text : AT(ADDR(.text) - LOAD_OFFSET) {
		*(.text .stub .text.* .gnu.linkonce.t.*)
	}

	PROVIDE(etext = .);	/* Define the 'etext' symbol to this value */

	.rodata : AT(ADDR(.rodata) - LOAD_OFFSET) {
		*(.rodata .rodata.* .gnu.linkonce.r.*)
	}

  /*
  .gcc_except_table : {
    *(.gcc_except_table)
  }

  .debug_gdb_scripts : {
    *(.debug_gdb_scripts)
  }
  */

	/* Include debugging information in kernel memory */
	.stab : AT(ADDR(.stab) - LOAD_OFFSET) {
		PROVIDE(__STAB_BEGIN__ = .);
		*(.stab);
		PROVIDE(__STAB_END__ = .);
		BYTE(0);		/* Force the linker to allocate space
				   for this section */
	}

	.stabstr : AT(ADDR(.stabstr) - LOAD_OFFSET) {
		PROVIDE(__STABSTR_BEGIN__ = .);
		*(.stabstr);
		PROVIDE(__STABSTR_END__ = .);
		BYTE(0)		/* Force the linker to allocate space
				   for this section */
	}

	/* Adjust the address for the data segment to the next page */
	. = ALIGN(0x1000);

	/* Conventionally, Unix linkers provide pseudo-symbols
	 * etext, edata, and end, at the end of the text, data, and bss.
	 * For the kernel mapping, we need the address at the beginning
	 * of the data section, but that's not one of the conventional
	 * symbols, because the convention started before there was a
	 * read-only rodata section between text and data. */
	PROVIDE(data = .);
	PROVIDE(__data = .);

	/* The data segment */
	.data : AT(ADDR(.data) - LOAD_OFFSET) {
		*(.data .data.*)
	}

	PROVIDE(edata = .);

	.bss : AT(ADDR(.bss) - LOAD_OFFSET) {
		*(.bss .bss.*)
	}

	PROVIDE(end = .);
	PROVIDE(__end = .);

	/DISCARD/ : {
		*(.eh_frame .note.GNU-stack)
	}
}
