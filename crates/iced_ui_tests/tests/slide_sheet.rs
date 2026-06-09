//! Snapshot tests for the [`iced_ui::SlideSheet`] widget.

use iced::widget::{column, text};
use iced_test::Error;
use iced_ui::Theme;
use iced_ui::slide_sheet::SlideSheet;
use iced_ui_tests::{Element, TALL_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    Dismiss,
}

#[test]
fn slide_sheet_collapsed() -> Result<(), Error> {
    // Collapsed slide sheet should only show the host content.
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let element = SlideSheet::new(host, "Sheet content")
        .expanded(false)
        .on_dismiss(Message::Dismiss)
        .drag_handle(true);

    assert_snapshot::<Message>("slide_sheet_collapsed", element, TALL_SIZE)
}
