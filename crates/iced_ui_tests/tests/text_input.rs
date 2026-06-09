//! Snapshot tests for the [`iced_ui::TextInput`] widget.

use iced::widget::row;
use iced_test::Error;
use iced_ui::text_input::TextInput;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    Changed(String),
}

#[test]
fn text_input_outlined_default() -> Result<(), Error> {
    let element = row![
        TextInput::new("Placeholder...", "")
            .on_input(Message::Changed)
            .width(iced::Length::Fixed(200.0))
    ]
    .padding(20);
    assert_snapshot::<Message>("text_input_outlined_default", element, DEFAULT_SIZE)
}

#[test]
fn text_input_filled_default() -> Result<(), Error> {
    let element = row![
        TextInput::new("Filled...", "")
            .on_input(Message::Changed)
            .variant(iced_ui::text_input::Variant::Filled)
            .width(iced::Length::Fixed(200.0))
    ]
    .padding(20);
    assert_snapshot::<Message>("text_input_filled_default", element, DEFAULT_SIZE)
}

#[test]
fn text_input_with_value() -> Result<(), Error> {
    let element = row![
        TextInput::new("Placeholder", "Hello world")
            .on_input(Message::Changed)
            .width(iced::Length::Fixed(200.0))
    ]
    .padding(20);
    assert_snapshot::<Message>("text_input_with_value", element, DEFAULT_SIZE)
}
