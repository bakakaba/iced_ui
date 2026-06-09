//! The [`Numeric`] trait and implementations for standard number
//! types.

use std::fmt::Display;
use std::ops::RangeInclusive;
use std::str::FromStr;

/// Trait for numeric types supported by [`NumberInput`](super::NumberInput).
///
/// Provides operations needed by the number input: arithmetic for
/// stepping, clamping to a range, display formatting, and
/// per-character validation to reject invalid keystrokes.
pub trait Numeric: Copy + PartialOrd + Display + FromStr + 'static {
    /// The additive identity.
    fn zero() -> Self;

    /// Adds two values.
    fn add(self, other: Self) -> Self;

    /// Subtracts `other` from `self`.
    fn sub(self, other: Self) -> Self;

    /// Clamps `self` to the given inclusive range.
    fn clamp_range(self, range: &RangeInclusive<Self>) -> Self;

    /// Returns `true` if the character `c` could appear in a valid
    /// representation of this type, given the current text buffer
    /// contents.
    ///
    /// This is used to reject invalid keystrokes in real time.
    fn is_valid_char(c: char, current: &str) -> bool;
}

macro_rules! impl_numeric_unsigned {
    ($($t:ty),+) => {
        $(
            impl Numeric for $t {
                fn zero() -> Self { 0 }
                fn add(self, other: Self) -> Self { self.saturating_add(other) }
                fn sub(self, other: Self) -> Self { self.saturating_sub(other) }
                fn clamp_range(self, range: &RangeInclusive<Self>) -> Self {
                    self.clamp(*range.start(), *range.end())
                }
                fn is_valid_char(c: char, _current: &str) -> bool {
                    c.is_ascii_digit()
                }
            }
        )+
    };
}

macro_rules! impl_numeric_signed {
    ($($t:ty),+) => {
        $(
            impl Numeric for $t {
                fn zero() -> Self { 0 }
                fn add(self, other: Self) -> Self { self.saturating_add(other) }
                fn sub(self, other: Self) -> Self { self.saturating_sub(other) }
                fn clamp_range(self, range: &RangeInclusive<Self>) -> Self {
                    self.clamp(*range.start(), *range.end())
                }
                fn is_valid_char(c: char, current: &str) -> bool {
                    if c.is_ascii_digit() {
                        return true;
                    }
                    // Allow leading minus sign
                    c == '-' && current.is_empty()
                }
            }
        )+
    };
}

macro_rules! impl_numeric_float {
    ($($t:ty),+) => {
        $(
            impl Numeric for $t {
                fn zero() -> Self { 0.0 }
                fn add(self, other: Self) -> Self { self + other }
                fn sub(self, other: Self) -> Self { self - other }
                fn clamp_range(self, range: &RangeInclusive<Self>) -> Self {
                    self.clamp(*range.start(), *range.end())
                }
                fn is_valid_char(c: char, current: &str) -> bool {
                    if c.is_ascii_digit() {
                        return true;
                    }
                    // Allow one decimal point
                    if c == '.' && !current.contains('.') {
                        return true;
                    }
                    // Allow leading minus sign
                    c == '-' && current.is_empty()
                }
            }
        )+
    };
}

impl_numeric_unsigned!(u8, u16, u32, u64);
impl_numeric_signed!(i8, i16, i32, i64);
impl_numeric_float!(f32, f64);
