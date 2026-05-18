//! Snapshot tests for the [`iced_ui::Chip`] widget.

use iced::widget::{row, text};
use iced_test::Error;
use iced_ui::Chip;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    Toggled,
    Pressed,
    Closed,
}

#[test]
fn chip_assist_default() -> Result<(), Error> {
    let element =
        row![Chip::assist(text("Add event").size(14)).on_press(Message::Pressed)].padding(20);
    assert_snapshot::<Message>("chip_assist_default", element, DEFAULT_SIZE)
}

#[test]
fn chip_filter_unselected() -> Result<(), Error> {
    let element = row![
        Chip::filter(text("Vegetarian").size(14))
            .selected(false)
            .on_press(Message::Toggled)
    ]
    .padding(20);
    assert_snapshot::<Message>("chip_filter_unselected", element, DEFAULT_SIZE)
}

#[test]
fn chip_filter_selected() -> Result<(), Error> {
    let element = row![
        Chip::filter(text("Vegetarian").size(14))
            .selected(true)
            .on_press(Message::Toggled)
    ]
    .padding(20);
    assert_snapshot::<Message>("chip_filter_selected", element, DEFAULT_SIZE)
}

#[test]
fn chip_input_default() -> Result<(), Error> {
    let element = row![
        Chip::input(text("John Doe").size(14))
            .on_press(Message::Pressed)
            .on_close(Message::Closed)
    ]
    .padding(20);
    assert_snapshot::<Message>("chip_input_default", element, DEFAULT_SIZE)
}

#[test]
fn chip_suggestion_default() -> Result<(), Error> {
    let element =
        row![Chip::suggestion(text("Quick reply").size(14)).on_press(Message::Pressed)].padding(20);
    assert_snapshot::<Message>("chip_suggestion_default", element, DEFAULT_SIZE)
}
