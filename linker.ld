/* Based on:
 * - https://github.com/cccriscv/mini-riscv-os/blob/master/10-SystemCall/os.ld
 * - https://github.com/skyzh/core-os-riscv/blob/master/kernel/src/kernel.ld
 */

OUTPUT_ARCH( "riscv" )

ENTRY( _start )

MEMORY {
	ram : ORIGIN = 0x80200000, LENGTH = 126M
}

PHDRS {
	text PT_LOAD;
	data PT_LOAD;
	bss PT_LOAD;
}

SECTIONS {
	. = ALIGN(4096);
	.text : {
		PROVIDE(_text_start = .);
		*(.text.init) *(.text .text.*)
		PROVIDE(_text_end = .);
	} >ram AT>ram :text


	. = ALIGN(4096);
	.rodata : {
		PROVIDE(_rodata_start = .);
		*(.rodata .rodata.*)
		PROVIDE(_rodata_end = .);
	} >ram AT>ram :text

	. = ALIGN(4096);
	.data : {
		PROVIDE(_data_start = .);
		*(.sdata .sdata.*) *(.data .data.*)
		PROVIDE(_data_end = .);
	} >ram AT>ram :data

	. = ALIGN(4096);
	.bss :{
		PROVIDE(_bss_start = .);
		*(.sbss .sbss.*) *(.bss .bss.*)
		PROVIDE(_bss_end = .);

		/* (Statically) allocate 64 MiB for allocator */
		. = ALIGN(4096);
		PROVIDE(_allocator_start = .);
		. = . + 0x4000000;
		PROVIDE(_allocator_end = .);
	} >ram AT>ram :bss
}
