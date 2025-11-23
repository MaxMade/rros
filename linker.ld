/* Based on:
 * - https://github.com/cccriscv/mini-riscv-os/blob/master/10-SystemCall/os.ld
 * - https://github.com/skyzh/core-os-riscv/blob/master/kernel/src/kernel.ld
 */

OUTPUT_ARCH( "riscv" )

ENTRY( _start )

ADDRESS_OFFSET = 0xffffffff00000000;

SECTIONS {
	. = 0x80200000 + ADDRESS_OFFSET;

	. = ALIGN(4096);
	.text : AT (ADDR (.text) - ADDRESS_OFFSET) {
		PROVIDE(_text_start = .);
		*(.text.init) *(.text .text.*)
		PROVIDE(_text_end = .);
	}

	. = ALIGN(4096);
	.rodata : AT (ADDR (.rodata) - ADDRESS_OFFSET) {
		PROVIDE(_rodata_start = .);
		*(.rodata .rodata.*)
		PROVIDE(_rodata_end = .);
	}

	. = ALIGN(4096);
	.data : AT (ADDR (.data) - ADDRESS_OFFSET) {
		PROVIDE(_data_start = .);
		*(.sdata .sdata.*) *(.data .data.*)
		PROVIDE(_data_end = .);
	}

	. = ALIGN(4096);
	.bss : AT (ADDR (.bss) - ADDRESS_OFFSET) {
		PROVIDE(_bss_start = .);
		*(.sbss .sbss.*) *(.bss .bss.*)

		/* (Statically) allocated 16 MiB for pages */
		. = ALIGN(4096);
		PROVIDE(_pages_start = .);
		. = . + 0x1000000;
		PROVIDE(_pages_end = .);

		PROVIDE(_bss_end = .);
	}
}
