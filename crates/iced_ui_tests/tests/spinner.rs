//! Snapshot tests for the [`iced_ui::Spinner`] widget.
//!
//! Each test renders the widget using its default configuration (per
//! the project's "Demo showcases defaults" rule) and compares the
//! result against a golden PNG reference image in `tests/snapshots/`.
//!
//! The spinner animates over time, but the first rendered frame (before
//! any `RedrawRequested` is observed) is pinned to phase 0, so these
//! snapshots are deterministic.

use iced::widget::row;
use iced_test::Error;
use iced_ui::Spinner;
use iced_ui::spinner::Color;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {}

#[test]
fn spinner_default() -> Result<(), Error> {
    let element = row![Spinner::new()].padding(20);
    assert_snapshot::<Message>("spinner_default", element, DEFAULT_SIZE)
}

#[test]
fn spinner_primary() -> Result<(), Error> {
    let element = row![Spinner::new().color(Color::Primary)].padding(20);
    assert_snapshot::<Message>("spinner_primary", element, DEFAULT_SIZE)
}

#[test]
fn spinner_success() -> Result<(), Error> {
    let element = row![Spinner::new().color(Color::Success)].padding(20);
    assert_snapshot::<Message>("spinner_success", element, DEFAULT_SIZE)
}

#[test]
fn spinner_warning() -> Result<(), Error> {
    let element = row![Spinner::new().color(Color::Warning)].padding(20);
    assert_snapshot::<Message>("spinner_warning", element, DEFAULT_SIZE)
}

#[test]
fn spinner_danger() -> Result<(), Error> {
    let element = row![Spinner::new().color(Color::Danger)].padding(20);
    assert_snapshot::<Message>("spinner_danger", element, DEFAULT_SIZE)
}
