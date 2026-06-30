//! A time input field with an hour/minute picker popup.
//!
//! [`TimeInput`] is a controlled text field for a [`chrono::NaiveTime`]
//! using the 24-hour `HH:MM` format. The trailing clock trigger opens
//! a popup with scrollable hour and minute columns; the value can also
//! be edited by typing (including minutes that are not on the
//! configured [`minute_step`](TimeInput::minute_step)).

use std::ops::RangeInclusive;

use chrono::{NaiveDate, NaiveTime};
use iced::{Element, Length};

use super::core::{Mode, Picker};
use super::style::Style;
use crate::text_input::style::Variant;
use crate::theme::Theme;

/// The fixed dummy date carried alongside time-only values.
fn dummy_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(1970, 1, 1).expect("valid epoch date")
}

/// A time input field with an hour/minute picker popup.
///
/// The consumer holds a [`NaiveTime`] value and receives a new one
/// through [`on_change`](Self::on_change) whenever a valid time is
/// typed or picked from the popup. Seconds are always zero.
pub struct TimeInput<'a, Message> {
    inner: Picker<'a, Message>,
}

impl<'a, Message> TimeInput<'a, Message> {
    /// Creates a new time input displaying the given time.
    pub fn new(value: NaiveTime) -> Self {
        Self {
            inner: Picker::new(Mode::Time, dummy_date().and_time(value)),
        }
    }

    /// Sets the handler called when the time changes.
    pub fn on_change(mut self, f: impl Fn(NaiveTime) -> Message + 'a) -> Self {
        self.inner = self.inner.on_change(Box::new(move |dt| f(dt.time())));
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

    /// Restricts the selectable times to the given inclusive range.
    ///
    /// Typed and picked values are clamped to the range on commit.
    pub fn range(mut self, range: RangeInclusive<NaiveTime>) -> Self {
        self.inner = self
            .inner
            .min(dummy_date().and_time(*range.start()))
            .max(dummy_date().and_time(*range.end()));
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

    /// Overrides the corner roundness of the input field, bypassing the
    /// theme's default for this widget. Accepts a
    /// [`Roundness`](crate::Roundness) token:
    /// [`Roundness::sx`](crate::Roundness::sx) scales the theme's
    /// roundness base, [`Roundness::px`](crate::Roundness::px) sets an
    /// absolute radius.
    pub fn roundness(mut self, roundness: crate::Roundness) -> Self {
        self.inner = self.inner.roundness(roundness);
        self
    }

    /// Sets the style of the picker popup.
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self {
        self.inner = self.inner.style(Box::new(style));
        self
    }
}

impl<'a, Message> From<TimeInput<'a, Message>> for Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    fn from(input: TimeInput<'a, Message>) -> Self {
        Element::new(input.inner)
    }
}
