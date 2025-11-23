//! Formatted Output.

use core::array;
use core::fmt::Arguments;
use core::fmt::Error;
use core::fmt::Write;
use core::hint;
use core::ptr;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

use crate::drivers::uart::UART;
use crate::kernel::cpu;
use crate::sync::per_core::PerCore;

const MSG_BUFFER_SIZE: usize = 512;
struct Formatter<'a> {
    buffer: &'a mut [u8; MSG_BUFFER_SIZE],
    len: &'a mut usize,
}

impl<'a> Write for Formatter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Calculate output length: Write `MSG_BUFFER_SIZE` characters at most!
        let remaining = MSG_BUFFER_SIZE.checked_sub(*self.len).unwrap();
        let len = usize::min(s.len(), remaining);

        // # Safety
        // For the sake over performance, copy data from `s` directly to `self.buffer` without any
        // additional bound checks. By calculating the minimum of `s.len()` and `MSG_BUFFER_SIZE`,
        // there will *never* occur an buffer overflow.
        unsafe { ptr::copy_nonoverlapping(s.as_ptr(), self.buffer.as_mut_ptr(), len) };

        // Save written length
        *self.len += len;

        Ok(())
    }
}

/// Global printer for formatted output.
pub struct Printer {
    /// Low priority message buffers (for each hart), e.g. during `epilogue`s.
    low_priority_msgs: PerCore<[u8; MSG_BUFFER_SIZE]>,

    /// Low priority message length (for each hart), e.g. during `epilogue`s.
    low_priority_lens: PerCore<usize>,

    /// High priority message buffers (for each hart), e.g. during `prologue`s.
    high_priority_msgs: PerCore<[u8; MSG_BUFFER_SIZE]>,

    /// High priority message length (for each hart), e.g. during `prologue`s.
    high_priority_lens: PerCore<usize>,

    ticket: AtomicUsize,

    serving: AtomicUsize,
}

impl Printer {
    /// Create a new printer instance.
    pub fn new() -> Self {
        Self {
            low_priority_msgs: PerCore::new_fn(|_| array::from_fn(|_| 0)),
            low_priority_lens: PerCore::new_copy(0),
            high_priority_msgs: PerCore::new_fn(|_| array::from_fn(|_| 0)),
            high_priority_lens: PerCore::new_copy(0),
            ticket: AtomicUsize::new(0),
            serving: AtomicUsize::new(0),
        }
    }

    /// Begin formatted output.
    pub fn write_fmt(&self, args: Arguments<'_>) -> Result<(), Error> {
        // Step 1: Check if output consists of a low or high priority message.
        //
        // If the interrupts are currently disabled, the output message is considered
        // high-priority. Otherwise, let's assume the low-priority
        let is_high_priority = !cpu::interrupts_enabled();

        // Step 2: Get the corresponding buffer
        //
        // # Safety
        // The members `high_priority_msgs`, `high_priority_lens`, `low_priority_msgs` and
        // `low_priority_lens` implement core-local storage for the respective message
        // buffers/lengths. Without potential rescheduling, it is safe to access core-local storage
        // using the given logical CPU ID.
        let (buffer, len) = match is_high_priority {
            true => unsafe {
                let buffer = self.high_priority_msgs.get_mut();
                let len = self.high_priority_lens.get_mut();
                (buffer, len)
            },
            false => unsafe {
                let buffer = self.low_priority_msgs.get_mut();
                let len = self.low_priority_lens.get_mut();
                (buffer, len)
            },
        };

        // Step 3: Write formatted messages to buffer
        *len = 0;
        let mut formatter = Formatter { buffer, len };
        formatter.write_fmt(args)?;

        // Step 4: Proceed with actual output using UART driver.
        let interrupt_flag = cpu::save_and_disable_interrupts();
        let ticket = self.ticket.fetch_add(1, Ordering::Relaxed);
        while ticket != self.serving.load(Ordering::Acquire) {
            hint::spin_loop();
        }
        for i in 0..*len {
            let _ = unsafe { UART.write_unchecked(buffer[i]) };
        }
        self.serving.fetch_add(1, Ordering::Release);
        cpu::restore_interrupts(interrupt_flag);

        Ok(())
    }
}
