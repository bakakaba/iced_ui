//! Snapshot tests for the [`iced_ui::BottomSheet`] widget.

use iced::widget::{column, text};
use iced_test::Error;
use iced_ui::Theme;
use iced_ui::bottom_sheet::BottomSheet;
use iced_ui_tests::{Element, TALL_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    Dismiss,
}

#[test]
fn bottom_sheet_collapsed() -> Result<(), Error> {
    // Collapsed bottom sheet should only show the host content.
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let element = BottomSheet::new(host, "Sheet content")
        .modal(true)
        .expanded(false)
        .on_dismiss(Message::Dismiss)
        .drag_handle(true);

    assert_snapshot::<Message>("bottom_sheet_collapsed", element, TALL_SIZE)
}
