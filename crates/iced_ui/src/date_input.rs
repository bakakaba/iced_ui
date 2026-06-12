//! A date input field with a calendar picker popup.
//!
//! [`DateInput`] is a controlled text field for a [`chrono::NaiveDate`]
//! using the ISO 8601 `YYYY-MM-DD` format. The trailing calendar
//! trigger opens a month-grid popup for picking a day with the mouse;
//! clicking the month or year in the popup header switches to a month
//! or year list for faster navigation. The value can also be edited by
//! typing.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::DateInput;
//!
//! let input = DateInput::new(self.start_date)
//!     .on_change(Message::StartDateChanged);
//! ```

pub use crate::date_time_core::style::{Style, StyleFn, default};

use std::ops::RangeInclusive;

use chrono::{NaiveDate, NaiveTime};
use iced::{Element, Length};

use crate::date_time_core::{Mode, Picker};
use crate::text_input::style::Variant;
use crate::theme::Theme;

/// A date input field with a calendar picker popup.
///
/// The consumer holds a [`NaiveDate`] value and receives a new one
/// through [`on_change`](Self::on_change) whenever a valid date is
/// typed or picked from the popup calendar.
pub struct DateInput<'a, Message> {
    inner: Picker<'a, Message>,
}

impl<'a, Message> DateInput<'a, Message> {
    /// Creates a new date input displaying the given date.
    pub fn new(value: NaiveDate) -> Self {
        Self {
            inner: Picker::new(Mode::Date, value.and_time(NaiveTime::MIN)),
        }
    }

    /// Sets the handler called when the date changes.
    pub fn on_change(mut self, f: impl Fn(NaiveDate) -> Message + 'a) -> Self {
        self.inner = self.inner.on_change(Box::new(move |dt| f(dt.date())));
        self
    }

    /// Restricts the selectable dates to the given inclusive range.
    ///
    /// Days outside the range are disabled in the popup calendar, and
    /// typed values are clamped on commit (Enter or blur).
    pub fn range(mut self, range: RangeInclusive<NaiveDate>) -> Self {
        self.inner = self
            .inner
            .min(range.start().and_time(NaiveTime::MIN))
            .max(range.end().and_time(NaiveTime::MIN));
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

impl<'a, Message> From<DateInput<'a, Message>> for Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    fn from(input: DateInput<'a, Message>) -> Self {
        Element::new(input.inner)
    }
}
