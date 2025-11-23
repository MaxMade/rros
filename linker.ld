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
		PROVIDE(__virt_text_start = .);
		*(.text.init) *(.text .text.*)
		PROVIDE(__virt_text_end = .);
	}

	. = ALIGN(4096);
	.rodata : AT (ADDR (.rodata) - ADDRESS_OFFSET) {
		PROVIDE(__virt_rodata_start = .);
		*(.rodata .rodata.*)
		PROVIDE(__virt_rodata_end = .);
	}

	. = ALIGN(4096);
	.data : AT (ADDR (.data) - ADDRESS_OFFSET) {
		PROVIDE(__virt_data_start = .);
		*(.sdata .sdata.*) *(.data .data.*)
		PROVIDE(__virt_data_end = .);
	}

	. = ALIGN(4096);
	.bss : AT (ADDR (.bss) - ADDRESS_OFFSET) {
		PROVIDE(__virt_bss_start = .);
		*(.sbss .sbss.*) *(.bss .bss.*)

		/* (Statically) allocated 16 MiB for pages */
		. = ALIGN(4096);
		PROVIDE(__virt_pages_start = .);
		. = . + 0x1000000;
		PROVIDE(__virt_pages_end = .);

		PROVIDE(__virt_bss_end = .);
	}

	PROVIDE(__phys_text_start = __virt_text_start - ADDRESS_OFFSET);
	PROVIDE(__phys_text_end = __virt_text_end - ADDRESS_OFFSET);
	PROVIDE(__phys_rodata_start = __virt_rodata_start - ADDRESS_OFFSET);
	PROVIDE(__phys_data_start = __virt_data_start - ADDRESS_OFFSET);
	PROVIDE(__phys_data_end = __virt_data_end - ADDRESS_OFFSET);
	PROVIDE(__phys_bss_start = __virt_bss_start - ADDRESS_OFFSET);
	PROVIDE(__phys_pages_start = __virt_pages_start - ADDRESS_OFFSET);
	PROVIDE(__phys_pages_end = __virt_pages_end - ADDRESS_OFFSET);
	PROVIDE(__phys_bss_end = __virt_bss_end - ADDRESS_OFFSET);
}
