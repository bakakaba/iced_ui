//! Snapshot tests for the [`iced_ui::IconButton`] widget.

use iced::widget::{row, text};
use iced_test::Error;
use iced_ui::icon_button::{self, IconButton};
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    Pressed,
}

#[test]
fn icon_button_standard_default() -> Result<(), Error> {
    let element = row![IconButton::new(text("X").size(18)).on_press(Message::Pressed)].padding(20);
    assert_snapshot::<Message>("icon_button_standard_default", element, DEFAULT_SIZE)
}

#[test]
fn icon_button_filled() -> Result<(), Error> {
    let element = row![
        IconButton::new(text("+").size(18))
            .variant(icon_button::Variant::Filled)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("icon_button_filled", element, DEFAULT_SIZE)
}

#[test]
fn icon_button_filled_tonal() -> Result<(), Error> {
    let element = row![
        IconButton::new(text("?").size(18))
            .variant(icon_button::Variant::FilledTonal)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("icon_button_filled_tonal", element, DEFAULT_SIZE)
}

#[test]
fn icon_button_outlined() -> Result<(), Error> {
    let element = row![
        IconButton::new(text("!").size(18))
            .variant(icon_button::Variant::Outlined)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("icon_button_outlined", element, DEFAULT_SIZE)
}

#[test]
fn icon_button_toggled_on() -> Result<(), Error> {
    let element = row![
        IconButton::new(text("*").size(18))
            .variant(icon_button::Variant::Filled)
            .toggled(true)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("icon_button_toggled_on", element, DEFAULT_SIZE)
}

#[test]
fn icon_button_toggled_off() -> Result<(), Error> {
    let element = row![
        IconButton::new(text("*").size(18))
            .variant(icon_button::Variant::Filled)
            .toggled(false)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("icon_button_toggled_off", element, DEFAULT_SIZE)
}
