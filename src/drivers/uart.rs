//! Driver for NS16550a UART.
//!
//! For more information, see:
//! - [The NS16550A: UART Design and Application
//! Considerations](https://mth.st/blog/riscv-qemu/AN-491.pdf)
//! - [(RISCV) RISC-V System, Booting, and
//! Interrupts](https://marz.utk.edu/my-courses/cosc562/riscv/)
use core::ptr;

use crate::boot::device_tree::dt::DeviceTree;
use crate::drivers::driver::Driver;

use crate::drivers::mmio::MMIOSpace;
use crate::kernel::address::PhysicalAddress;
use crate::kernel::address::VirtualAddress;

use crate::drivers::driver::DriverError;
use crate::sync::level::Level;
use crate::sync::level::LevelDriver;
use crate::sync::level::LevelInitialization;
use crate::sync::ticketlock::TicketlockDriver;

/// Global Uart instance.
pub static UART: Uart = Uart::new();

/// Register offsets (in bytes) relative to start of configuration space.
#[derive(Debug)]
enum RegisterOffset {
    /// Receive Holding Register.
    ///
    /// # Bit Field
    /// * Bits [0, 7]: Data bits
    RHR = 0,
    /// Interrupt Enable Register.
    ///
    /// # Bit Field
    /// * Bit 0: RHRI (See [ISRBitOffset])
    /// * Bit 1: THRI (See [ISRBitOffset])
    /// * Bit 2: RLSI (See [ISRBitOffset])
    /// * Bit 3: Mea (See [ISRBitOffset])
    /// * Bits [4, 7]: Unused
    IER = 1,
    /// Interrupt Status Register.
    ///
    /// # Bit Field
    /// Bit 0: Flags if an interrupt has occurred
    /// Bits [1, 2]: Interrupt cause
    /// Bits [3, 7]: Unused
    ISR = 2,
    /// Line Control Register.
    ///
    /// # Bit Field
    /// * Bits [0, 1]: Number of data bits (See [DataBits])
    /// * Bit 2: Number of stop bits (See [StopBits]):
    /// * Bits [3, 5]: Parity mode (See [ParityMode])
    /// * Bit 6: Break Condition
    /// * Bit 7: DLR Access Enabled.
    LCR = 3,
    /// Modem Control Register.
    ///
    /// # Bit Field
    /// Bit 0: Data terminal ready line
    /// Bit 1: Request to send line
    /// Bit 2: GPO1 (General Purpose Output 1)
    /// Bit 3: GPO2 (General Purpose Output 2)
    /// Bit 4: Echo test
    /// Bits [5, 7]: Unused
    MCR = 4,
    /// Line Status Register.
    ///
    /// # Bit Field
    /// Bit 0: Set if RHR contains a character
    /// Bit 1: Overrun error
    /// Bit 2: Parity error
    /// Bit 3: Framing error
    /// Bit 4: Break condition
    /// Bit 5: Transmit buffer empty
    /// Bit 6: Transmitter empty
    /// Bit 7: Unused
    LSR = 5,
    /// Modem Status Register.
    ///
    /// Bit 0: CTS (Clear To Send) line has changed
    /// Bit 1: DSR (Data Set Ready) has changed
    /// Bit 2: RI (Ring Indicator) has been set
    /// Bit 3: CD (Carrier Detect) has changed
    /// Bit 4: Value of CTS
    /// Bit 5: Value of DSR
    /// Bit 6: Value of RI
    /// Bit 7: Value of CD
    MSR = 6,
    /// Scratch Pad Register.
    SPR = 7,
}

/// Parity mode.
#[derive(Debug)]
enum ParityMode {
    /// No parity.
    No = 0b000,
    /// Odd parity.
    Odd = 0b001,
    /// Even parity.
    Even = 0b011,
    /// Mark parity.
    Mark = 0b101,
    /// Space parity.
    Space = 0b111,
}

/// Number of stop bits.
#[derive(Debug)]
enum StopBits {
    One = 0b0,
    Two = 0b1,
}

/// Number of data bits.
#[derive(Debug)]
enum DataBits {
    /// Five data bits.
    Five = 0b00,
    /// Six data bits.
    Six = 0b01,
    /// Seven data bits.
    Seven = 0b10,
    /// Eight data bits.
    Eight = 0b11,
}

/// Bit offset (within) `ISR` register to configure interrupts.
#[derive(Debug)]
enum ISRBitOffset {
    /// Receive Holding Register Interrupt.
    RHRI = 0,
    /// Transmit Holding Register Interrupt.
    THRI = 1,
    /// Receive Line Status Interrupt.
    RLSI = 2,
    /// Modem Status Interrupt.
    MSI = 3,
}

/// Bit offset (within) `LCR` register.
#[derive(Debug)]
enum LCRBitOffset {
    /// Offset for number of data bits
    DataBits = 0,
    /// Offset for number of stop bits
    StopBits = 2,
    /// Offset for number of stop bits
    ParityMode = 3,
    /// Offset for break condition
    BreakCondition = 6,
    /// Offset for DLR access enabled
    DLREnabled = 7,
}

/// Bit offset (within) `LSR` register.
#[derive(Debug)]
enum LSRBitOffset {
    /// Offset for Set if RHR contains a character
    RHRNonEmpty = 0,
    /// Offset for Overrun error
    OverrunError = 1,
    /// Offset for Parity error
    ParityError = 2,
    /// Offset for Framing error
    FramingError = 3,
    /// Offset for Break condition
    BreakCondition = 4,
    /// Offset for Transmit buffer empty
    TransmitBufferEmpty = 5,
    /// Offset for Transmitter empty
    TransmitterEmpty = 6,
}

/// Driver for UART NS16550a.
struct UARTNS16550a {
    config_space: MMIOSpace,
    clock_freq: usize,
}

impl UARTNS16550a {
    /// Create new UART NS16550a driver.
    pub const fn new() -> Self {
        /* Create driver with invalid mmio_space */
        unsafe {
            UARTNS16550a {
                config_space: MMIOSpace::new(VirtualAddress::new(ptr::null_mut()), 0),
                clock_freq: 0,
            }
        }
    }

    /// Get `Reveive Holding Register`.
    fn get_rhr(&self) -> u8 {
        self.config_space
            .load(RegisterOffset::RHR as usize)
            .unwrap()
    }

    /// Set `Reveive Holding Register`.
    fn set_rhr(&mut self, value: u8) {
        self.config_space
            .store(RegisterOffset::RHR as usize, value)
            .unwrap()
    }

    /// Configure number of data/stop bits and parity mode.
    fn configure_transmition(
        &mut self,
        data_bits: DataBits,
        stop_bits: StopBits,
        parity_mode: ParityMode,
    ) {
        let mut lcr: u8 = self
            .config_space
            .load(RegisterOffset::LCR as usize)
            .unwrap();
        lcr &= 0b11000000;
        lcr |= (data_bits as u8) << LCRBitOffset::DataBits as usize;
        lcr |= (stop_bits as u8) << LCRBitOffset::StopBits as usize;
        lcr |= (parity_mode as u8) << LCRBitOffset::ParityMode as usize;
        self.config_space
            .store(RegisterOffset::LCR as usize, lcr)
            .unwrap();
    }

    /// Enable DLR access.
    fn enable_dlr_access(&mut self) {
        let mut lcr: u8 = self
            .config_space
            .load(RegisterOffset::LCR as usize)
            .unwrap();
        lcr |= 1 << LCRBitOffset::DLREnabled as usize;
        self.config_space
            .store(RegisterOffset::LCR as usize, lcr)
            .unwrap();
    }

    /// Disable DLR access.
    fn disable_dlr_access(&mut self) {
        let mut lcr: u8 = self
            .config_space
            .load(RegisterOffset::LCR as usize)
            .unwrap();
        lcr &= !(1 << LCRBitOffset::DLREnabled as usize);
        self.config_space
            .store(RegisterOffset::LCR as usize, lcr)
            .unwrap();
    }

    /// Configure number of data/stop bits and parity mode.
    ///
    /// * `baud_rate`: Required baud rate (must be divisor of 115200).
    fn configure_baudrate(&mut self, baud_rate: u32) {
        /* Enable DLR access */
        self.enable_dlr_access();

        /* Configure divisor */
        let divider = 0x1c200u32.checked_div(baud_rate).unwrap();
        let lower_devicer = divider as u16;
        let upper_devicer = (divider >> 16) as u16;

        self.config_space
            .store(RegisterOffset::RHR as usize, lower_devicer)
            .unwrap();
        self.config_space
            .store(RegisterOffset::IER as usize, upper_devicer)
            .unwrap();

        /* Disable DLR access */
        self.disable_dlr_access();
    }

    /// Disable `Receive Holding Interrupt`.
    fn disable_rhri(&mut self) {
        let mut value: u8 = self
            .config_space
            .load(RegisterOffset::IER as usize)
            .unwrap();
        value &= !(1 << (ISRBitOffset::RHRI as u8));
        self.config_space
            .store(RegisterOffset::IER as usize, value)
            .unwrap();
    }

    /// Enable `Receive Holding Interrupt`.
    fn enbale_rhri(&mut self) {
        let mut value: u8 = self
            .config_space
            .load(RegisterOffset::IER as usize)
            .unwrap();
        value |= 1 << (ISRBitOffset::RHRI as u8);
        self.config_space
            .store(RegisterOffset::IER as usize, value)
            .unwrap();
    }

    /// Disable `Transmit Holding Register Interrupt`.
    fn disable_thri(&mut self) {
        let mut value: u8 = self
            .config_space
            .load(RegisterOffset::IER as usize)
            .unwrap();
        value &= !(1 << (ISRBitOffset::THRI as u8));
        self.config_space
            .store(RegisterOffset::IER as usize, value)
            .unwrap();
    }

    /// Enable `Transmit Holding Register Interrupt`.
    fn enbale_thri(&mut self) {
        let mut value: u8 = self
            .config_space
            .load(RegisterOffset::IER as usize)
            .unwrap();
        value |= 1 << (ISRBitOffset::THRI as u8);
        self.config_space
            .store(RegisterOffset::IER as usize, value)
            .unwrap();
    }

    /// Disable `Receive Line Status Interrupt`.
    fn disable_rlsi(&mut self) {
        let mut value: u8 = self
            .config_space
            .load(RegisterOffset::IER as usize)
            .unwrap();
        value &= !(1 << (ISRBitOffset::RLSI as u8));
        self.config_space
            .store(RegisterOffset::IER as usize, value)
            .unwrap();
    }

    /// Enable `Receive Line Status Interrupt`.
    fn enbale_rlsi(&mut self) {
        let mut value: u8 = self
            .config_space
            .load(RegisterOffset::IER as usize)
            .unwrap();
        value |= 1 << (ISRBitOffset::RLSI as u8);
        self.config_space
            .store(RegisterOffset::IER as usize, value)
            .unwrap();
    }

    /// Disable `Modem Status Interrupt`.
    fn disable_msi(&mut self) {
        let mut value: u8 = self
            .config_space
            .load(RegisterOffset::IER as usize)
            .unwrap();
        value &= !(1 << (ISRBitOffset::MSI as u8));
        self.config_space
            .store(RegisterOffset::IER as usize, value)
            .unwrap();
    }

    /// Enable `Modem Status Interrupt`.
    fn enbale_msi(&mut self) {
        let mut value: u8 = self
            .config_space
            .load(RegisterOffset::IER as usize)
            .unwrap();
        value |= 1 << (ISRBitOffset::MSI as u8);
        self.config_space
            .store(RegisterOffset::IER as usize, value)
            .unwrap();
    }
}

/// Locked version of driver for UART NS16550a.
pub struct Uart(TicketlockDriver<UARTNS16550a>);

impl Uart {
    /// Create a new `Uart` instance.
    pub const fn new() -> Self {
        Uart(TicketlockDriver::new(UARTNS16550a::new()))
    }
}

impl Driver for Uart {
    fn initiailize(
        token: LevelInitialization,
    ) -> Result<LevelInitialization, (DriverError, LevelInitialization)>
    where
        Self: Sized,
    {
        // Search device tree for node describing ns16550a
        let device_tree = DeviceTree::get_dt();
        let device = match device_tree.get_node_by_compatible_property("ns16550a") {
            Some(device) => device,
            None => return Err((DriverError::NonCompatibleDevice, token)),
        };

        /* Get locked driver */
        let mut driver = UART.0.init_lock(token);

        /* Get address and size of configuration space */
        let reg_property = match device.property_iter().filter(|p| p.name == "reg").next() {
            Some(reg_property) => reg_property,
            None => {
                let token = driver.init_unlock();
                return Err((DriverError::NonCompatibleDevice, token));
            }
        };
        let (raw_address, raw_length) = match reg_property.into_addr_length_iter().next() {
            Some((raw_address, raw_length)) => (raw_address, raw_length),
            None => {
                let token = driver.init_unlock();
                return Err((DriverError::NonCompatibleDevice, token));
            }
        };
        let _phys_addres = PhysicalAddress::from(raw_address as *mut u8);
        let size = raw_length;

        /* TODO: Convert physical address to virtual address */
        let virt_address = VirtualAddress::from(raw_address as *mut u8);

        /* Create configuration space */
        let config_space = unsafe { MMIOSpace::new(virt_address, size) };
        driver.config_space = config_space;

        /* Read clock frequency */
        let clock_freq = match device
            .property_iter()
            .filter(|p| p.name == "clock-frequency")
            .next()
        {
            Some(clock_freq) => clock_freq,
            None => {
                let token = driver.init_unlock();
                return Err((DriverError::NonCompatibleDevice, token));
            }
        };
        let clock_freq = match clock_freq.get_value() {
            crate::boot::device_tree::property::PropertyValue::U32(clock_freq) => {
                clock_freq as usize
            }
            crate::boot::device_tree::property::PropertyValue::U64(clock_freq) => {
                clock_freq as usize
            }
            _ => {
                let token = driver.init_unlock();
                return Err((DriverError::NonCompatibleDevice, token));
            }
        };
        driver.clock_freq = clock_freq;

        /* Disable all interrupts */
        driver.disable_rhri();
        driver.disable_thri();
        driver.disable_rlsi();
        driver.disable_msi();

        /* Configure baudrate */
        driver.configure_baudrate(115200);

        /* Configure output */
        driver.configure_transmition(DataBits::Eight, StopBits::One, ParityMode::No);

        /* Enable interrupts */
        driver.enbale_rhri();

        /* Unlock driver */
        let token = driver.init_unlock();

        return Ok(token);
    }
}

impl Uart {
    /// Write single byte `value` using serial interface.
    pub fn write(
        &self,
        value: u8,
        token: LevelDriver,
    ) -> Result<LevelDriver, (DriverError, LevelDriver)> {
        /* Get locked driver */
        let (mut driver, token) = self.0.lock(token);

        /* Wait for device to finish previous transmission */
        loop {
            let lsr: u8 = driver
                .config_space
                .load(RegisterOffset::LSR as usize)
                .unwrap();
            if (lsr & (1 << LSRBitOffset::TransmitBufferEmpty as usize)) != 0 {
                break;
            }
        }

        driver
            .config_space
            .store(RegisterOffset::RHR as usize, value)
            .unwrap();

        let token = driver.unlock(token);

        return Ok(token);
    }

    /// Write single byte `value` using serial interface without Level validation.
    pub unsafe fn write_unchecked(&self, value: u8) -> Result<(), DriverError> {
        let token = LevelDriver::create();
        match self.write(value, token) {
            Ok(_) => return Ok(()),
            Err((error, _)) => return Err(error),
        }
    }

    fn read(&self) -> Result<u8, DriverError> {
        todo!()
    }
}
