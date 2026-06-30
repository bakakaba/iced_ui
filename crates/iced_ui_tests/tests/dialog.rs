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

fn host() -> Element<'static, Message, Theme> {
    column![text("Host content").size(16)].padding(20).into()
}

#[test]
fn dialog_closed() -> Result<(), Error> {
    // When closed, only the host content is rendered.
    let element = Dialog::new(host())
        .title(text("Confirm action").size(20))
        .content(text("Are you sure you want to proceed?"))
        .confirm(Message::Confirmed)
        .dismiss(Message::Dismissed)
        .on_scrim_press(Message::ScrimPressed)
        .open(false);

    assert_snapshot::<Message>("dialog_closed", element, TALL_SIZE)
}

#[test]
fn dialog_open() -> Result<(), Error> {
    // Title + content + default OK/Cancel action buttons.
    let element = Dialog::new(host())
        .title(text("Confirm action").size(20))
        .content(text("Are you sure you want to proceed?"))
        .confirm(Message::Confirmed)
        .dismiss(Message::Dismissed)
        .on_scrim_press(Message::ScrimPressed)
        .open(true);

    assert_snapshot::<Message>("dialog_open", element, TALL_SIZE)
}

#[test]
fn dialog_content_only() -> Result<(), Error> {
    // Content alone composes the whole dialog: no title, no actions.
    let element = Dialog::new(host())
        .content(text("Just some content, no title and no action buttons."))
        .on_scrim_press(Message::ScrimPressed)
        .open(true);

    assert_snapshot::<Message>("dialog_content_only", element, TALL_SIZE)
}

#[test]
fn dialog_confirm_only() -> Result<(), Error> {
    // A single default confirm button (no dismiss).
    let element = Dialog::new(host())
        .title(text("Heads up").size(20))
        .content(text("Only a confirm button is shown."))
        .confirm(Message::Confirmed)
        .open(true);

    assert_snapshot::<Message>("dialog_confirm_only", element, TALL_SIZE)
}
