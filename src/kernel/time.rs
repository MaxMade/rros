//! Time-related abstractions

use core::fmt::Debug;
use core::fmt::Display;
use core::hash::Hash;
use core::ops::Add;
use core::ops::AddAssign;
use core::ops::Div;
use core::ops::DivAssign;
use core::ops::Mul;
use core::ops::MulAssign;
use core::ops::Rem;
use core::ops::RemAssign;
use core::ops::Sub;
use core::ops::SubAssign;

const NANO_SECONDS_FACTOR: usize = 1;
const MILLI_SECONDS_FACTOR: usize = 1 * 1000;
const MICRO_SECONDS_FACTOR: usize = 1 * 1000 * 1000;
const SECONDS_FACTOR: usize = 1 * 1000 * 1000 * 1000;
const MINUTES_FACTOR: usize = 60 * 1000 * 1000 * 1000;
const HOURS_FACTOR: usize = 60 * 60 * 1000 * 1000 * 1000;
const DAYS_FACTOR: usize = 24 * 60 * 60 * 1000 * 1000 * 1000;

/// Generic time unit trait
pub trait TimeUnits:
    Debug
    + Display
    + Copy
    + Clone
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Hash
    + Add
    + AddAssign
    + Div<usize>
    + DivAssign<usize>
    + Mul<usize>
    + MulAssign<usize>
    + Rem
    + RemAssign
    + Sub
    + SubAssign
{
}

/// Type for handling seconds.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeUnit<const FACTOR: usize>(usize);

impl<const FACTOR: usize> TimeUnit<FACTOR> {
    /// Convert different time formats
    pub fn convert<const OTHER: usize>(self) -> TimeUnit<OTHER> {
        match usize::cmp(&FACTOR, &OTHER) {
            core::cmp::Ordering::Less => {
                // Option 1: Try more precise conversion
                if let Some(value_ns) = self.0.checked_mul(FACTOR) {
                    return TimeUnit::<OTHER>(value_ns / OTHER);
                }

                // Option 2: Less precise conversion
                return TimeUnit::<OTHER>(self.0 * (OTHER / FACTOR));
            }
            core::cmp::Ordering::Greater => {
                // Option 1: Try more precise conversion
                if let Some(value_ns) = self.0.checked_mul(FACTOR) {
                    return TimeUnit::<OTHER>(value_ns / OTHER);
                }

                // Option 2: Less precise conversion
                return TimeUnit::<OTHER>(self.0 / (FACTOR / OTHER));
            }
            core::cmp::Ordering::Equal => TimeUnit::<OTHER>(self.0),
        }
    }
}

impl<const FACTOR: usize> TimeUnit<FACTOR> {
    /// Create a new [`TimeUnit`].
    pub const fn new(time: usize) -> Self {
        Self(time)
    }

    /// Get underlying (raw) value.
    pub const fn raw(self) -> usize {
        self.0
    }
}

impl<const FACTOR: usize> Display for TimeUnit<FACTOR> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match FACTOR {
            NANO_SECONDS_FACTOR => write!(f, "{} ns", self.0),
            MILLI_SECONDS_FACTOR => write!(f, "{} us", self.0),
            MICRO_SECONDS_FACTOR => write!(f, "{} ms", self.0),
            SECONDS_FACTOR => write!(f, "{} s", self.0),
            MINUTES_FACTOR => write!(f, "{} min", self.0),
            HOURS_FACTOR => write!(f, "{} h", self.0),
            DAYS_FACTOR => write!(f, "{} d", self.0),
            _ => write!(f, "{}", self.0),
        }
    }
}

impl<const FACTOR: usize> Add for TimeUnit<FACTOR> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<const FACTOR: usize> AddAssign for TimeUnit<FACTOR> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<const FACTOR: usize> Sub for TimeUnit<FACTOR> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<const FACTOR: usize> SubAssign for TimeUnit<FACTOR> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl<const FACTOR: usize> Mul<usize> for TimeUnit<FACTOR> {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl<const FACTOR: usize> MulAssign<usize> for TimeUnit<FACTOR> {
    fn mul_assign(&mut self, rhs: usize) {
        self.0 *= rhs;
    }
}

impl<const FACTOR: usize> Div<usize> for TimeUnit<FACTOR> {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl<const FACTOR: usize> DivAssign<usize> for TimeUnit<FACTOR> {
    fn div_assign(&mut self, rhs: usize) {
        self.0 /= rhs;
    }
}

impl<const FACTOR: usize> Rem for TimeUnit<FACTOR> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl<const FACTOR: usize> RemAssign for TimeUnit<FACTOR> {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
    }
}

/// Specialized type for days.
pub type Day = TimeUnit<DAYS_FACTOR>;

/// Specialized type for hours.
pub type Hour = TimeUnit<HOURS_FACTOR>;

/// Specialized type for minutes.
pub type Minute = TimeUnit<MINUTES_FACTOR>;

/// Specialized type for seconds.
pub type Second = TimeUnit<SECONDS_FACTOR>;

/// Specialized type for milliseconds.
pub type MilliSecond = TimeUnit<MICRO_SECONDS_FACTOR>;

/// Specialized type for microseconds.
pub type MicroSecond = TimeUnit<MILLI_SECONDS_FACTOR>;

/// Specialized type for nanoseconds.
pub type NanoSecond = TimeUnit<NANO_SECONDS_FACTOR>;
