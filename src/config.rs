//! Tailor your system simply by using `config.yaml` configuration file.
//!
//! # Caution
//! This file is auto-generated using the `build.rs` script! Do not change any values here, as those
//! might be overwritten by the next invocation of `cargo build`.

/// Maximum number of supported CPUs.
pub const MAX_CPU_NUM: usize = 8;
/// Page size.
pub const PAGE_SIZE: usize = 4096;
/// Log level filtering used by [`printk`](crate::kernel::printer::Printer)..
pub const LOG_LEVEL: crate::kernel::printer::LogLevel = crate::kernel::printer::LogLevel::Trace;
