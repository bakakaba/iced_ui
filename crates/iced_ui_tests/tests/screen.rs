//! Snapshot tests for the [`iced_ui::Screen`] widget.

use iced::widget::{column, text};
use iced_test::Error;
use iced_ui::screen::{Mode, Screen};
use iced_ui_tests::{DEFAULT_SIZE, TALL_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {}

#[test]
fn screen_desktop_default() -> Result<(), Error> {
    let element = column![Screen::new(column![text("Desktop").size(14)].padding(8))].padding(10);

    assert_snapshot::<Message>("screen_desktop_default", element, DEFAULT_SIZE)
}

#[test]
fn screen_mobile_landscape() -> Result<(), Error> {
    let element = column![
        Screen::new(column![text("Landscape").size(14)].padding(8)).mode(Mode::MobileLandscape)
    ]
    .padding(10);

    assert_snapshot::<Message>("screen_mobile_landscape", element, DEFAULT_SIZE)
}

#[test]
fn screen_mobile_portrait() -> Result<(), Error> {
    let element = column![
        Screen::new(column![text("Portrait").size(14)].padding(8)).mode(Mode::MobilePortrait)
    ]
    .padding(10);

    assert_snapshot::<Message>("screen_mobile_portrait", element, TALL_SIZE)
}
