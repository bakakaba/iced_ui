//! Snapshot tests for the [`iced_ui::Divider`] widget.

use iced::widget::{column, row, text};
use iced_test::Error;
use iced_ui::Divider;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {}

#[test]
fn divider_horizontal_default() -> Result<(), Error> {
    let element = column![text("above"), Divider::horizontal(), text("below"),]
        .spacing(8)
        .padding(20);

    assert_snapshot::<Message>("divider_horizontal_default", element, DEFAULT_SIZE)
}

#[test]
fn divider_vertical_default() -> Result<(), Error> {
    let element = row![text("left"), Divider::vertical(), text("right"),]
        .spacing(8)
        .padding(20)
        .height(iced::Length::Fixed(80.0));

    assert_snapshot::<Message>("divider_vertical_default", element, DEFAULT_SIZE)
}
