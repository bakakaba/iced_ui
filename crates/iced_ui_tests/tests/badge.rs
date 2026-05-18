//! Snapshot tests for the [`iced_ui::Badge`] widget.
//!
//! Each test renders the widget using its default configuration (per
//! the project's "Demo showcases defaults" rule) and compares the
//! result against a golden PNG reference image in `tests/snapshots/`.

use iced::widget::{row, text};
use iced_test::Error;
use iced_ui::Badge;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {}

#[test]
fn badge_dot_default() -> Result<(), Error> {
    let element = row![Badge::dot(text("Mail").size(16))].padding(20);
    assert_snapshot::<Message>("badge_dot_default", element, DEFAULT_SIZE)
}

#[test]
fn badge_count_default() -> Result<(), Error> {
    let element = row![Badge::count(text("Inbox").size(16), 5)].padding(20);
    assert_snapshot::<Message>("badge_count_default", element, DEFAULT_SIZE)
}

#[test]
fn badge_count_overflow() -> Result<(), Error> {
    // Counts greater than `max` should render as "<max>+".
    let element = row![Badge::count(text("Notifications").size(16), 1234).max(999)].padding(20);
    assert_snapshot::<Message>("badge_count_overflow", element, DEFAULT_SIZE)
}
