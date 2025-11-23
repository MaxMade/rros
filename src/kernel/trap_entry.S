.global __trap_entry

.set CONTEXT_SIZE, (36 * 8)
.set CONTEXT_OFFSET_X1, (0 * 8)
.set CONTEXT_OFFSET_X2, (1 * 8)
.set CONTEXT_OFFSET_X3, (2 * 8)
.set CONTEXT_OFFSET_X4, (3 * 8)
.set CONTEXT_OFFSET_X5, (4 * 8)
.set CONTEXT_OFFSET_X6, (5 * 8)
.set CONTEXT_OFFSET_X7, (6 * 8)
.set CONTEXT_OFFSET_X8, (7 * 8)
.set CONTEXT_OFFSET_X9, (8 * 8)
.set CONTEXT_OFFSET_X10, (9 * 8)
.set CONTEXT_OFFSET_X11, (10 * 8)
.set CONTEXT_OFFSET_X12, (11 * 8)
.set CONTEXT_OFFSET_X13, (12 * 8)
.set CONTEXT_OFFSET_X14, (13 * 8)
.set CONTEXT_OFFSET_X15, (14 * 8)
.set CONTEXT_OFFSET_X16, (15 * 8)
.set CONTEXT_OFFSET_X17, (16 * 8)
.set CONTEXT_OFFSET_X18, (17 * 8)
.set CONTEXT_OFFSET_X19, (18 * 8)
.set CONTEXT_OFFSET_X20, (19 * 8)
.set CONTEXT_OFFSET_X21, (20 * 8)
.set CONTEXT_OFFSET_X22, (21 * 8)
.set CONTEXT_OFFSET_X23, (22 * 8)
.set CONTEXT_OFFSET_X24, (23 * 8)
.set CONTEXT_OFFSET_X25, (24 * 8)
.set CONTEXT_OFFSET_X26, (25 * 8)
.set CONTEXT_OFFSET_X27, (26 * 8)
.set CONTEXT_OFFSET_X28, (27 * 8)
.set CONTEXT_OFFSET_X29, (28 * 8)
.set CONTEXT_OFFSET_X30, (29 * 8)
.set CONTEXT_OFFSET_X31, (30 * 8)
.set CONTEXT_OFFSET_SSTATUS, (31 * 8)
.set CONTEXT_OFFSET_SSCRATCH, (32 * 8)
.set CONTEXT_OFFSET_SEPC, (33 * 8)
.set CONTEXT_OFFSET_SCAUSE, (34 * 8)
.set CONTEXT_OFFSET_STVAL, (35 * 8)

.section .text

// Low-level trap handler
.align 4
__trap_entry:
	// Determine the execution mode at which the trap was triggered: If the
	// trap interrupted the supervisor mode, the sscratch register will contain
	// the value NULL. Otherwise, a pointer to core local storage will be
	// present. Hereby, the logicial CPU ID will be encoded within the lower
	// byte(s) of this address.

	// Atomically swap tp and sscratch
	csrrw tp, sscratch, tp

	/// Check if tp is zero => Trapped instruction in supervisor mode
	beqz tp, .trap_from_supervisor_mode

.trap_from_user_mode:
	// TODO: Perform stack switch

	// TODO: Save (user) integer-regsiter

	// TODO: Save (user) control/status register

	// TODO: Calculate logical CPU ID based on tp register (swapped with sscratch)

	// TODO: Update sscratch

	j .handle_trap

.trap_from_supervisor_mode:
	// Restore tp
	csrrw tp, sscratch, tp

	// Save (supervisor) integer-regsiter
	addi sp, sp, -CONTEXT_SIZE
	sd x1, CONTEXT_OFFSET_X1(sp) // x2 (or sp) will be saved with its orignal value
	addi x1, sp, CONTEXT_SIZE
	sd x1, CONTEXT_OFFSET_X2(sp)
	sd x3, CONTEXT_OFFSET_X3(sp)
	sd x4, CONTEXT_OFFSET_X4(sp)
	sd x5, CONTEXT_OFFSET_X5(sp)
	sd x6, CONTEXT_OFFSET_X6(sp)
	sd x7, CONTEXT_OFFSET_X7(sp)
	sd x8, CONTEXT_OFFSET_X8(sp)
	sd x9, CONTEXT_OFFSET_X9(sp)
	sd x10, CONTEXT_OFFSET_X10(sp)
	sd x11, CONTEXT_OFFSET_X11(sp)
	sd x12, CONTEXT_OFFSET_X12(sp)
	sd x13, CONTEXT_OFFSET_X13(sp)
	sd x14, CONTEXT_OFFSET_X14(sp)
	sd x15, CONTEXT_OFFSET_X15(sp)
	sd x16, CONTEXT_OFFSET_X16(sp)
	sd x17, CONTEXT_OFFSET_X17(sp)
	sd x18, CONTEXT_OFFSET_X18(sp)
	sd x19, CONTEXT_OFFSET_X19(sp)
	sd x20, CONTEXT_OFFSET_X20(sp)
	sd x21, CONTEXT_OFFSET_X21(sp)
	sd x22, CONTEXT_OFFSET_X22(sp)
	sd x23, CONTEXT_OFFSET_X23(sp)
	sd x24, CONTEXT_OFFSET_X24(sp)
	sd x25, CONTEXT_OFFSET_X25(sp)
	sd x26, CONTEXT_OFFSET_X26(sp)
	sd x27, CONTEXT_OFFSET_X27(sp)
	sd x28, CONTEXT_OFFSET_X28(sp)
	sd x29, CONTEXT_OFFSET_X29(sp)
	sd x30, CONTEXT_OFFSET_X30(sp)
	sd x31, CONTEXT_OFFSET_X31(sp)

	// Save (supervisor) control/status-registers
	csrr a0, sstatus
	sd a0, CONTEXT_OFFSET_SSTATUS(sp)

	sd x0, CONTEXT_OFFSET_SSCRATCH(sp)

	csrr a0, sepc
	sd a0, CONTEXT_OFFSET_SEPC(sp)

	csrr a0, scause
	sd a0, CONTEXT_OFFSET_SCAUSE(sp)

	csrr a0, stval
	sd a0, CONTEXT_OFFSET_STVAL(sp)

	// Pass stack pointer as argument to trap_handler
	mv a0, sp
	xor a1, a1, a1

.handle_trap:
	// Start actual trap handling
	call trap_handler

	// Restore control/status registers
	ld a0, CONTEXT_OFFSET_STVAL(sp)
	csrw stval, a0

	ld a0, CONTEXT_OFFSET_SCAUSE(sp)
	csrw scause, a0

	ld a0, CONTEXT_OFFSET_SEPC(sp)
	csrw sepc, a0

	ld a0, CONTEXT_OFFSET_SSCRATCH(sp)
	csrw sscratch, a0

	ld a0, CONTEXT_OFFSET_SSTATUS(sp)
	csrw sstatus, a0

	// Restore (supervisor) integer-regsiter
	ld x31, CONTEXT_OFFSET_X31(sp)
	ld x30, CONTEXT_OFFSET_X30(sp)
	ld x29, CONTEXT_OFFSET_X29(sp)
	ld x27, CONTEXT_OFFSET_X27(sp)
	ld x28, CONTEXT_OFFSET_X28(sp)
	ld x26, CONTEXT_OFFSET_X26(sp)
	ld x25, CONTEXT_OFFSET_X25(sp)
	ld x24, CONTEXT_OFFSET_X24(sp)
	ld x23, CONTEXT_OFFSET_X23(sp)
	ld x22, CONTEXT_OFFSET_X22(sp)
	ld x21, CONTEXT_OFFSET_X21(sp)
	ld x20, CONTEXT_OFFSET_X20(sp)
	ld x19, CONTEXT_OFFSET_X19(sp)
	ld x18, CONTEXT_OFFSET_X18(sp)
	ld x17, CONTEXT_OFFSET_X17(sp)
	ld x16, CONTEXT_OFFSET_X16(sp)
	ld x15, CONTEXT_OFFSET_X15(sp)
	ld x14, CONTEXT_OFFSET_X14(sp)
	ld x13, CONTEXT_OFFSET_X13(sp)
	ld x12, CONTEXT_OFFSET_X12(sp)
	ld x11, CONTEXT_OFFSET_X11(sp)
	ld x10, CONTEXT_OFFSET_X10(sp)
	ld x9, CONTEXT_OFFSET_X9(sp)
	ld x8, CONTEXT_OFFSET_X8(sp)
	ld x7, CONTEXT_OFFSET_X7(sp)
	ld x6, CONTEXT_OFFSET_X6(sp)
	ld x5, CONTEXT_OFFSET_X5(sp)
	ld x4, CONTEXT_OFFSET_X4(sp)
	ld x3, CONTEXT_OFFSET_X3(sp)
	ld x1, CONTEXT_OFFSET_X1(sp)

	// XXX: x2 (or sp) must be restore at the very end!
	ld x2, CONTEXT_OFFSET_X2(sp)

	sret
