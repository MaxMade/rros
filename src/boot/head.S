// Size of boot stack
.equ STACK_SIZE, (16 * 4096)

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

	// Jump to Rust World
	j kernel_ap_init

.setup_stack:
	// Calculate stack offset
	li t0, STACK_SIZE
	addi t0, t0, -16

	// Setup stack for boot cpu
	la sp, boot_stack
	add sp, sp, t0

	// Jump to Rust World
	j kernel_init

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
