//! Snapshot tests for the [`iced_ui::Chip`] widget.

use iced::widget::row;
use iced_test::Error;
use iced_ui::Chip;
use iced_ui::chip::{ChipColor, ChipSize};
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    Toggled,
    Removed,
}

#[test]
fn chip_default() -> Result<(), Error> {
    let element = row![Chip::new("Add event").on_toggle(Message::Toggled)].padding(20);
    assert_snapshot::<Message>("chip_default", element, DEFAULT_SIZE)
}

#[test]
fn chip_color_primary() -> Result<(), Error> {
    let element = row![
        Chip::new("Vegetarian")
            .color(ChipColor::Primary)
            .on_toggle(Message::Toggled)
    ]
    .padding(20);
    assert_snapshot::<Message>("chip_color_primary", element, DEFAULT_SIZE)
}

#[test]
fn chip_color_custom() -> Result<(), Error> {
    let element = row![
        Chip::new("Custom")
            .color(ChipColor::Custom(iced::Color::from_rgb(0.8, 0.2, 0.5)))
            .on_toggle(Message::Toggled)
    ]
    .padding(20);
    assert_snapshot::<Message>("chip_color_custom", element, DEFAULT_SIZE)
}

#[test]
fn chip_size_sm() -> Result<(), Error> {
    let element = row![
        Chip::new("Small")
            .size(ChipSize::Sm)
            .on_toggle(Message::Toggled)
    ]
    .padding(20);
    assert_snapshot::<Message>("chip_size_sm", element, DEFAULT_SIZE)
}

#[test]
fn chip_size_lg() -> Result<(), Error> {
    let element = row![
        Chip::new("Large")
            .size(ChipSize::Lg)
            .on_toggle(Message::Toggled)
    ]
    .padding(20);
    assert_snapshot::<Message>("chip_size_lg", element, DEFAULT_SIZE)
}

#[test]
fn chip_removable() -> Result<(), Error> {
    let element = row![
        Chip::new("John Doe")
            .on_toggle(Message::Toggled)
            .on_remove(Message::Removed)
    ]
    .padding(20);
    assert_snapshot::<Message>("chip_removable", element, DEFAULT_SIZE)
}
