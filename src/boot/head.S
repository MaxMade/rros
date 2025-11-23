// Size of boot stack
.equ STACK_SIZE, (16 * 4096)

// Size of a page table
.equ PT_SIZE, (4096)

// Size of a page-table entry
.equ PTE_SIZE, (8)

.global _start

.section .text.init

// Kernel startup entry
_start:
	// Mask all interrupts. See 4.1.3 Supervisor Interrupt Registers (sip and sie)
	csrw sie, zero
	csrw sip, zero

	// Load logical CPU ID
	la t0, cpu_counter
	li t1, 1
	amoadd.w a2, t1, (t0)

	// First CPU will bootstrap system
	beq a2, zero, .setup_stack

	// Calculate stack offset based on fetched cpu_counter value
	addi t0, a2, 1
	li t1, STACK_SIZE
	mul t0, t0, t1
	addi t0, t0, -16

	// Setup stack for application cpu(s)
	la sp, boot_stack
	add sp, sp, t0

	// XXX: Fix stack pointer for higher-half mapping
	li t1, -1
	slli t1, t1, 32
	or sp, sp, t1

	// Enable 39-bit paging
	la t0, pte
	srai t0, t0, 12
	li t1, -1
	slli t1, t1, 63
	or t0, t0, t1 // t0 = (pte >> 12) | (0x8 << 60)
	csrw satp, t0

	// Jump to Rust World
	la t0, kernel_ap_init

	// XXX: Fix function pointer for higher-half mapping
	li t1, -1
	slli t1, t1, 32
	or t0, t0, t1
	jalr t0

.setup_stack:
	// Calculate stack offset
	li t0, STACK_SIZE
	addi t0, t0, -16

	// Setup stack for boot cpu
	la sp, boot_stack
	add sp, sp, t0

	// XXX: Fix stack pointer for higher-half mapping
	li t1, -1
	slli t1, t1, 32
	or sp, sp, t1

	// Zero .bss
	la t0, __virt_bss_start
	mv t1, t0
	la t2, __virt_bss_end
.clear_bss_head:
	beq t1, t2, .clear_bss_out
	sb x0, (t1)
	addi t1, t1, 1
	j .clear_bss_head
.clear_bss_out:

	// Create identity mapping for lower 4 GiB
	// Load address of pte
	la t0, pte

	// Update physical page number (0x0000 00000 0000 0000) and protection bits (+RWX -UG)
	li t1, 0xf
	sd t1, (0 * PTE_SIZE)(t0)

	li t1, (0x10000000 | 0xf)
	// Update physical page number (0x0000 00000 4000 0000) and protection bits (+RWX -UG)
	sd t1, (1 * PTE_SIZE)(t0)

	li t1, (0x20000000 | 0xf)
	// Update physical page number (0x0000 00000 b000 0000) and protection bits (+RWX -UG)
	sd t1, (2 * PTE_SIZE)(t0)

	li t1, (0x30000000 | 0xf)
	// Update physical page number (0x0000 00000 c000 0000) and protection bits (+RWX -UG)
	sd t1, (3 * PTE_SIZE)(t0)

	// Update physical page number (0x0000 00000 0000 0000) and protection bits (+RWX -UG)
	li t2, 508 * PTE_SIZE
	add t2, t0, t2
	li t1, 0xf
	sd t1, (0 * PTE_SIZE)(t2)

	li t1, (0x10000000 | 0xf)
	// Update physical page number (0x0000 00000 4000 0000) and protection bits (+RWX -UG)
	sd t1, (1 * PTE_SIZE)(t2)

	li t1, (0x20000000 | 0xf)
	// Update physical page number (0x0000 00000 b000 0000) and protection bits (+RWX -UG)
	sd t1, (2 * PTE_SIZE)(t2)

	li t1, (0x30000000 | 0xf)
	// Update physical page number (0x0000 00000 c000 0000) and protection bits (+RWX -UG)
	sd t1, (3 * PTE_SIZE)(t2)

	// Calculate: pte / 4096
	srai t0, t0, 12

	// Calculate: pte | (0x8 << 60)
	li t1, -1
	slli t1, t1, 63
	or t0, t0, t1

	// Enable 39-bit paging
	csrw satp, t0

	// Load device tree size to register a2
	lw a2, 4(a1)
	srliw t0, a2, 8
	lui t1, 16
	addiw t1, t1, -256
	and t0, t0, t1
	srliw t1, a2, 24
	or t0, t0, t1
	slli t1, a2, 8
	lui a3, 4080
	and t1, t1, a3
	slliw a2, a2, 24
	or a2, a2, t1
	or a2, a2, t0
	add a2, a2, 0x28

	// Jump to Rust World
	la t0, kernel_init

	// XXX: Fix function pointer for higher-half mapping
	li t1, -1
	slli t1, t1, 32
	or t0, t0, t1
	jalr t0

// Endless loop for panicking within early boot
_startup_panic:
	wfi
    j _startup_panic

.section .data

.align 4
boot_stack:
    .skip STACK_SIZE * MAX_CPU_NUM

.align 4
cpu_counter:
	.word 0

.section .bss

.align 12
pte:
    .skip PT_SIZE
