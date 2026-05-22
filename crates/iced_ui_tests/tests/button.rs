//! Snapshot tests for the [`iced_ui::Button`] widget.

use iced::widget::{row, text};
use iced_test::Error;
use iced_ui::button::{Button, ButtonSize, Variant};
use iced_ui_tests::{DEFAULT_SIZE, WIDE_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    Pressed,
}

// --- Solid variant (default) ---

#[test]
fn button_solid_sm() -> Result<(), Error> {
    let element = row![
        Button::new(text("Click"))
            .size(ButtonSize::Sm)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("button_solid_sm", element, DEFAULT_SIZE)
}

#[test]
fn button_solid_md() -> Result<(), Error> {
    let element = row![Button::new(text("Click")).on_press(Message::Pressed)].padding(20);
    assert_snapshot::<Message>("button_solid_md", element, DEFAULT_SIZE)
}

#[test]
fn button_solid_lg() -> Result<(), Error> {
    let element = row![
        Button::new(text("Click"))
            .size(ButtonSize::Lg)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("button_solid_lg", element, DEFAULT_SIZE)
}

// --- Outline variant ---

#[test]
fn button_outline_md() -> Result<(), Error> {
    let element = row![
        Button::new(text("Click"))
            .variant(Variant::Outline)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("button_outline_md", element, DEFAULT_SIZE)
}

#[test]
fn button_outline_lg() -> Result<(), Error> {
    let element = row![
        Button::new(text("Click"))
            .variant(Variant::Outline)
            .size(ButtonSize::Lg)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("button_outline_lg", element, DEFAULT_SIZE)
}

// --- Ghost variant ---

#[test]
fn button_ghost_md() -> Result<(), Error> {
    let element = row![
        Button::new(text("Click"))
            .variant(Variant::Ghost)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("button_ghost_md", element, DEFAULT_SIZE)
}

#[test]
fn button_ghost_lg() -> Result<(), Error> {
    let element = row![
        Button::new(text("Click"))
            .variant(Variant::Ghost)
            .size(ButtonSize::Lg)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("button_ghost_lg", element, DEFAULT_SIZE)
}

// --- Disabled ---

#[test]
fn button_solid_disabled() -> Result<(), Error> {
    let element = row![
        Button::new(text("Click"))
            .enabled(false)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("button_solid_disabled", element, DEFAULT_SIZE)
}

#[test]
fn button_outline_disabled() -> Result<(), Error> {
    let element = row![
        Button::new(text("Click"))
            .variant(Variant::Outline)
            .enabled(false)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("button_outline_disabled", element, DEFAULT_SIZE)
}

#[test]
fn button_ghost_disabled() -> Result<(), Error> {
    let element = row![
        Button::new(text("Click"))
            .variant(Variant::Ghost)
            .enabled(false)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("button_ghost_disabled", element, DEFAULT_SIZE)
}

// --- All sizes in a row ---

#[test]
fn button_all_sizes() -> Result<(), Error> {
    let element = row![
        Button::new(text("SM"))
            .size(ButtonSize::Sm)
            .on_press(Message::Pressed),
        Button::new(text("MD"))
            .size(ButtonSize::Md)
            .on_press(Message::Pressed),
        Button::new(text("LG"))
            .size(ButtonSize::Lg)
            .on_press(Message::Pressed),
    ]
    .spacing(8)
    .padding(20);
    assert_snapshot::<Message>("button_all_sizes", element, WIDE_SIZE)
}
