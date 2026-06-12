//! A combined date-and-time input field with a picker popup.
//!
//! [`DateTimeInput`] is a controlled text field for a
//! [`chrono::NaiveDateTime`] using the `YYYY-MM-DD HH:MM` format. The
//! trailing trigger opens a single popup combining a month calendar
//! with scrollable hour and minute columns; the value can also be
//! edited by typing.

pub use crate::date_time_core::style::{Style, StyleFn, default};

use std::ops::RangeInclusive;

use chrono::{NaiveDate, NaiveDateTime};
use iced::{Element, Length};

use crate::date_time_core::{Mode, Picker};
use crate::text_input::style::Variant;
use crate::theme::Theme;

/// A combined date-and-time input field with a picker popup.
///
/// The consumer holds a [`NaiveDateTime`] value and receives a new one
/// through [`on_change`](Self::on_change) whenever a valid date-time
/// is typed or picked from the popup. Seconds are always zero.
pub struct DateTimeInput<'a, Message> {
    inner: Picker<'a, Message>,
}

impl<'a, Message> DateTimeInput<'a, Message> {
    /// Creates a new date-time input displaying the given value.
    pub fn new(value: NaiveDateTime) -> Self {
        Self {
            inner: Picker::new(Mode::DateTime, value),
        }
    }

    /// Sets the handler called when the date-time changes.
    pub fn on_change(mut self, f: impl Fn(NaiveDateTime) -> Message + 'a) -> Self {
        self.inner = self.inner.on_change(Box::new(f));
        self
    }

    /// Restricts the selectable values to the given inclusive range.
    ///
    /// Days fully outside the range are disabled in the popup
    /// calendar, and typed or picked values are clamped on commit.
    pub fn range(mut self, range: RangeInclusive<NaiveDateTime>) -> Self {
        self.inner = self.inner.min(*range.start()).max(*range.end());
        self
    }

    /// Overrides the date marked as "today" in the popup calendar.
    ///
    /// By default the system's local date is used. Override it when
    /// the application has its own notion of the current date (or for
    /// deterministic tests).
    pub fn today(mut self, today: NaiveDate) -> Self {
        self.inner = self.inner.today(today);
        self
    }

    /// Sets the step between the minutes offered in the popup's
    /// minute column (default: `1`).
    ///
    /// The step is clamped to `1..=59`. Off-step minutes can still be
    /// entered by typing.
    pub fn minute_step(mut self, step: u32) -> Self {
        self.inner = self.inner.minute_step(step);
        self
    }

    /// Sets the width of the input.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width.into());
        self
    }

    /// Sets the visual variant (outlined or filled).
    pub fn variant(mut self, variant: Variant) -> Self {
        self.inner = self.inner.variant(variant);
        self
    }

    /// Sets the style of the picker popup.
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.inner = self.inner.style(Box::new(style));
        self
    }
}

impl<'a, Message> From<DateTimeInput<'a, Message>> for Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    fn from(input: DateTimeInput<'a, Message>) -> Self {
        Element::new(input.inner)
    }
}
