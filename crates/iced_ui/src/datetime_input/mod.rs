//! Date, time, and combined date-time input fields.
//!
//! This module groups three conceptually related widgets that share a
//! single internal engine:
//!
//! - [`DateInput`] — a [`chrono::NaiveDate`] field (`YYYY-MM-DD`) with
//!   a calendar picker popup.
//! - [`TimeInput`] — a [`chrono::NaiveTime`] field (`HH:MM`, 24-hour)
//!   with an hour/minute picker popup.
//! - [`DateTimeInput`] — a [`chrono::NaiveDateTime`] field
//!   (`YYYY-MM-DD HH:MM`) combining both pickers in one popup.
//!
//! All three are thin public wrappers around the same internal
//! `Picker` widget: a text field (wrapping iced's built-in
//! `text_input`, following the
//! [`NumberInput`](crate::NumberInput) architecture) with a trailing
//! trigger that opens a popup overlay (following the
//! [`ColorPicker`](crate::ColorPicker) architecture). The popup shows a
//! calendar grid, an hour/minute grid, or both, depending on the
//! internal mode. Values are carried internally as
//! [`chrono::NaiveDateTime`]; the wrappers convert from/to `NaiveDate`
//! and `NaiveTime`.
//!
//! The popup panel is styled through the shared [`Style`], [`StyleFn`],
//! and [`default`] style function exposed by this module.

mod core;
mod date;
mod datetime;
mod time;

pub(crate) mod grid;
pub mod style;

pub use date::DateInput;
pub use datetime::DateTimeInput;
pub use style::{Style, StyleFn, default};
pub use time::TimeInput;
