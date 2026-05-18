//! Snapshot tests for the [`iced_ui::Snackbar`] widget.

use iced::widget::{column, text};
use iced_test::Error;
use iced_ui::Theme;
use iced_ui::snackbar::Snackbar;
use iced_ui_tests::{Element, TALL_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    Dismiss,
}

#[test]
fn snackbar_hidden() -> Result<(), Error> {
    // When not visible, only the host content is rendered.
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let element = Snackbar::new(host)
        .message("Item has been archived.")
        .action("Undo", Message::Dismiss)
        .on_dismiss(Message::Dismiss)
        .visible(false);

    assert_snapshot::<Message>("snackbar_hidden", element, TALL_SIZE)
}

#[test]
fn snackbar_visible() -> Result<(), Error> {
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let element = Snackbar::new(host)
        .message("Item has been archived.")
        .action("Undo", Message::Dismiss)
        .on_dismiss(Message::Dismiss)
        .visible(true);

    assert_snapshot::<Message>("snackbar_visible", element, TALL_SIZE)
}
