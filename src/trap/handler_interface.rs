//! Rusty Trap Entry.

use crate::kernel::cpu::Register;

use crate::kernel::cpu::SCause;
use crate::kernel::cpu::SScratch;
use crate::kernel::cpu::SStatus;
use crate::kernel::cpu::STVal;
use crate::kernel::cpu::SEPC;
use crate::sync::level::Level;
use crate::sync::level::LevelPrologue;
use crate::trap::cause::Interrupt;
use crate::trap::cause::Trap;
use crate::trap::handlers::TrapHandlers;
use crate::trap::intc::INTERRUPT_CONTROLLER;

/// Context object passed by low-level (assembly) trap entry.
pub struct TrapContext([u64; 36]);

impl TrapContext {
    /// Get register `x1` from [`TrapContext`]
    pub fn get_x1(&self) -> Register {
        Register::new(self.0[0])
    }

    /// Get register `x2` from [`TrapContext`]
    pub fn get_x2(&self) -> Register {
        Register::new(self.0[1])
    }

    /// Get register `x3` from [`TrapContext`]
    pub fn get_x3(&self) -> Register {
        Register::new(self.0[2])
    }

    /// Get register `x4` from [`TrapContext`]
    pub fn get_x4(&self) -> Register {
        Register::new(self.0[3])
    }

    /// Get register `x5` from [`TrapContext`]
    pub fn get_x5(&self) -> Register {
        Register::new(self.0[4])
    }

    /// Get register `x6` from [`TrapContext`]
    pub fn get_x6(&self) -> Register {
        Register::new(self.0[5])
    }

    /// Get register `x7` from [`TrapContext`]
    pub fn get_x7(&self) -> Register {
        Register::new(self.0[6])
    }

    /// Get register `x8` from [`TrapContext`]
    pub fn get_x8(&self) -> Register {
        Register::new(self.0[7])
    }

    /// Get register `x9` from [`TrapContext`]
    pub fn get_x9(&self) -> Register {
        Register::new(self.0[8])
    }

    /// Get register `x10` from [`TrapContext`]
    pub fn get_x10(&self) -> Register {
        Register::new(self.0[9])
    }

    /// Get register `x11` from [`TrapContext`]
    pub fn get_x11(&self) -> Register {
        Register::new(self.0[10])
    }

    /// Get register `x12` from [`TrapContext`]
    pub fn get_x12(&self) -> Register {
        Register::new(self.0[11])
    }

    /// Get register `x13` from [`TrapContext`]
    pub fn get_x13(&self) -> Register {
        Register::new(self.0[12])
    }

    /// Get register `x14` from [`TrapContext`]
    pub fn get_x14(&self) -> Register {
        Register::new(self.0[13])
    }

    /// Get register `x15` from [`TrapContext`]
    pub fn get_x15(&self) -> Register {
        Register::new(self.0[14])
    }

    /// Get register `x16` from [`TrapContext`]
    pub fn get_x16(&self) -> Register {
        Register::new(self.0[15])
    }

    /// Get register `x17` from [`TrapContext`]
    pub fn get_x17(&self) -> Register {
        Register::new(self.0[16])
    }

    /// Get register `x18` from [`TrapContext`]
    pub fn get_x18(&self) -> Register {
        Register::new(self.0[17])
    }

    /// Get register `x19` from [`TrapContext`]
    pub fn get_x19(&self) -> Register {
        Register::new(self.0[18])
    }

    /// Get register `x20` from [`TrapContext`]
    pub fn get_x20(&self) -> Register {
        Register::new(self.0[19])
    }

    /// Get register `x21` from [`TrapContext`]
    pub fn get_x21(&self) -> Register {
        Register::new(self.0[20])
    }

    /// Get register `x22` from [`TrapContext`]
    pub fn get_x22(&self) -> Register {
        Register::new(self.0[21])
    }

    /// Get register `x23` from [`TrapContext`]
    pub fn get_x23(&self) -> Register {
        Register::new(self.0[22])
    }

    /// Get register `x24` from [`TrapContext`]
    pub fn get_x24(&self) -> Register {
        Register::new(self.0[23])
    }

    /// Get register `x25` from [`TrapContext`]
    pub fn get_x25(&self) -> Register {
        Register::new(self.0[24])
    }

    /// Get register `x26` from [`TrapContext`]
    pub fn get_x26(&self) -> Register {
        Register::new(self.0[25])
    }

    /// Get register `x27` from [`TrapContext`]
    pub fn get_x27(&self) -> Register {
        Register::new(self.0[26])
    }

    /// Get register `x28` from [`TrapContext`]
    pub fn get_x28(&self) -> Register {
        Register::new(self.0[27])
    }

    /// Get register `x29` from [`TrapContext`]
    pub fn get_x29(&self) -> Register {
        Register::new(self.0[28])
    }

    /// Get register `x30` from [`TrapContext`]
    pub fn get_x30(&self) -> Register {
        Register::new(self.0[29])
    }

    /// Get register `x31` from [`TrapContext`]
    pub fn get_x31(&self) -> Register {
        Register::new(self.0[30])
    }

    /// Get register `sstatus` from [`TrapContext`]
    pub fn get_sstatus(&self) -> SStatus {
        SStatus::new(self.0[31])
    }

    /// Get register `sscratch` from [`TrapContext`]
    pub fn get_sscratch(&self) -> SScratch {
        SScratch::new(self.0[32])
    }

    /// Get register `sepc` from [`TrapContext`]
    pub fn get_sepc(&self) -> SEPC {
        SEPC::new(self.0[33])
    }

    /// Get register `scause` from [`TrapContext`]
    pub fn get_scause(&self) -> SCause {
        SCause::new(self.0[34])
    }

    /// Get register `stval` from [`TrapContext`]
    pub fn get_stval(&self) -> STVal {
        STVal::new(self.0[35])
    }

    /// Set register `x1` of [`TrapContext`].
    pub fn set_x1(&mut self, reg: Register) {
        self.0[0] = reg.raw();
    }

    /// Set register `x2` of [`TrapContext`].
    pub fn set_x2(&mut self, reg: Register) {
        self.0[1] = reg.raw();
    }

    /// Set register `x3` of [`TrapContext`].
    pub fn set_x3(&mut self, reg: Register) {
        self.0[2] = reg.raw();
    }

    /// Set register `x4` of [`TrapContext`].
    pub fn set_x4(&mut self, reg: Register) {
        self.0[3] = reg.raw();
    }

    /// Set register `x5` of [`TrapContext`].
    pub fn set_x5(&mut self, reg: Register) {
        self.0[4] = reg.raw();
    }

    /// Set register `x6` of [`TrapContext`].
    pub fn set_x6(&mut self, reg: Register) {
        self.0[5] = reg.raw();
    }

    /// Set register `x7` of [`TrapContext`].
    pub fn set_x7(&mut self, reg: Register) {
        self.0[6] = reg.raw();
    }

    /// Set register `x8` of [`TrapContext`].
    pub fn set_x8(&mut self, reg: Register) {
        self.0[7] = reg.raw();
    }

    /// Set register `x9` of [`TrapContext`].
    pub fn set_x9(&mut self, reg: Register) {
        self.0[8] = reg.raw();
    }

    /// Set register `x10` of [`TrapContext`].
    pub fn set_x10(&mut self, reg: Register) {
        self.0[9] = reg.raw();
    }

    /// Set register `x11` of [`TrapContext`].
    pub fn set_x11(&mut self, reg: Register) {
        self.0[10] = reg.raw();
    }

    /// Set register `x12` of [`TrapContext`].
    pub fn set_x12(&mut self, reg: Register) {
        self.0[11] = reg.raw();
    }

    /// Set register `x13` of [`TrapContext`].
    pub fn set_x13(&mut self, reg: Register) {
        self.0[12] = reg.raw();
    }

    /// Set register `x14` of [`TrapContext`].
    pub fn set_x14(&mut self, reg: Register) {
        self.0[13] = reg.raw();
    }

    /// Set register `x15` of [`TrapContext`].
    pub fn set_x15(&mut self, reg: Register) {
        self.0[14] = reg.raw();
    }

    /// Set register `x16` of [`TrapContext`].
    pub fn set_x16(&mut self, reg: Register) {
        self.0[15] = reg.raw();
    }

    /// Set register `x17` of [`TrapContext`].
    pub fn set_x17(&mut self, reg: Register) {
        self.0[16] = reg.raw();
    }

    /// Set register `x18` of [`TrapContext`].
    pub fn set_x18(&mut self, reg: Register) {
        self.0[17] = reg.raw();
    }

    /// Set register `x19` of [`TrapContext`].
    pub fn set_x19(&mut self, reg: Register) {
        self.0[18] = reg.raw();
    }

    /// Set register `x20` of [`TrapContext`].
    pub fn set_x20(&mut self, reg: Register) {
        self.0[19] = reg.raw();
    }

    /// Set register `x21` of [`TrapContext`].
    pub fn set_x21(&mut self, reg: Register) {
        self.0[20] = reg.raw();
    }

    /// Set register `x22` of [`TrapContext`].
    pub fn set_x22(&mut self, reg: Register) {
        self.0[21] = reg.raw();
    }

    /// Set register `x23` of [`TrapContext`].
    pub fn set_x23(&mut self, reg: Register) {
        self.0[22] = reg.raw();
    }

    /// Set register `x24` of [`TrapContext`].
    pub fn set_x24(&mut self, reg: Register) {
        self.0[23] = reg.raw();
    }

    /// Set register `x25` of [`TrapContext`].
    pub fn set_x25(&mut self, reg: Register) {
        self.0[24] = reg.raw();
    }

    /// Set register `x26` of [`TrapContext`].
    pub fn set_x26(&mut self, reg: Register) {
        self.0[25] = reg.raw();
    }

    /// Set register `x27` of [`TrapContext`].
    pub fn set_x27(&mut self, reg: Register) {
        self.0[26] = reg.raw();
    }

    /// Set register `x28` of [`TrapContext`].
    pub fn set_x28(&mut self, reg: Register) {
        self.0[27] = reg.raw();
    }

    /// Set register `x29` of [`TrapContext`].
    pub fn set_x29(&mut self, reg: Register) {
        self.0[28] = reg.raw();
    }

    /// Set register `x30` of [`TrapContext`].
    pub fn set_x30(&mut self, reg: Register) {
        self.0[29] = reg.raw();
    }

    /// Set register `x31` of [`TrapContext`].
    pub fn set_x31(&mut self, reg: Register) {
        self.0[30] = reg.raw();
    }

    /// Set register `sstatus` of [`TrapContext`].
    pub fn set_sstatus(&mut self, sstatus: SStatus) {
        self.0[31] = sstatus.raw();
    }

    /// Set register `sscratch` of [`TrapContext`].
    pub fn set_sscratch(&mut self, sscratch: SScratch) {
        self.0[31] = sscratch.raw();
    }

    /// Set register `sepc` of [`TrapContext`].
    pub fn set_sepc(&mut self, sepc: SEPC) {
        self.0[31] = sepc.raw();
    }

    /// Set register `scause` of [`TrapContext`].
    pub fn set_scause(&mut self, scause: SCause) {
        self.0[31] = scause.raw();
    }

    /// Set register `stval` of [`TrapContext`].
    pub fn set_stval(&mut self, stval: STVal) {
        self.0[31] = stval.raw();
    }
}

#[no_mangle]
extern "C" fn trap_handler(state: *mut TrapContext, user: usize) {
    // Create PROLOGUE token
    let token = unsafe { LevelPrologue::create() };

    // Create reference to register
    let state = unsafe { state.as_mut().unwrap() };

    // Check origin of trap
    assert!(user == 0, "Currently, no user traps are supported!");

    // Get scause
    let sscause = state.get_scause();

    // Get more generic abstraction of cause
    let trap = Trap::from(sscause);
    let (trap, token) = match trap {
        Trap::Interrupt(Interrupt::ExternalInterrupt) => {
            let (interrupt, token) = INTERRUPT_CONTROLLER.source(token);
            (Trap::Interrupt(interrupt), token)
        }
        Trap::Interrupt(_) => (trap, token),
        Trap::Exception(_) => (trap, token),
    };

    // Get corresponding handler
    let (handler, token) = TrapHandlers::get(trap, token);

    // Execute prologue
    let (epilogue_required, token) = handler.prologue(token);

    // Send end of interrupt if necessary
    let token = match trap {
        Trap::Interrupt(interrupt) => INTERRUPT_CONTROLLER.end_of_interrupt(interrupt, token),
        Trap::Exception(_) => token,
    };

    // Mark epilogue as pending (if necessary)
    let token = match epilogue_required {
        true => TrapHandlers::enqueue(trap, token),
        false => token,
    };

    // Execute pending epilogues
    todo!("Execute pending epilogues");
}
