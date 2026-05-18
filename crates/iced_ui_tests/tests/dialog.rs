//! Snapshot tests for the [`iced_ui::Dialog`] widget.

use iced::widget::{column, text};
use iced_test::Error;
use iced_ui::Theme;
use iced_ui::dialog::Dialog;
use iced_ui_tests::{Element, TALL_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    Confirmed,
    Dismissed,
    ScrimPressed,
}

#[test]
fn dialog_closed() -> Result<(), Error> {
    // When closed, only the host content is rendered.
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let element = Dialog::new(host)
        .title("Confirm action")
        .body("Are you sure you want to proceed?")
        .confirm("OK", Message::Confirmed)
        .dismiss("Cancel", Message::Dismissed)
        .on_scrim_press(Message::ScrimPressed)
        .open(false);

    assert_snapshot::<Message>("dialog_closed", element, TALL_SIZE)
}

#[test]
fn dialog_open() -> Result<(), Error> {
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let element = Dialog::new(host)
        .title("Confirm action")
        .body("Are you sure you want to proceed?")
        .confirm("OK", Message::Confirmed)
        .dismiss("Cancel", Message::Dismissed)
        .on_scrim_press(Message::ScrimPressed)
        .open(true);

    assert_snapshot::<Message>("dialog_open", element, TALL_SIZE)
}
