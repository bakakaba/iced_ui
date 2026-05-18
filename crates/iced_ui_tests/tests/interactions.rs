//! Interaction tests demonstrating non-snapshot behavioral testing.
//!
//! These tests exercise widgets via `Simulator` clicks and key taps,
//! and assert on the resulting message stream rather than the rendered
//! pixels. They guarantee that user input is correctly translated into
//! application messages without depending on a golden image.

use iced::widget::{row, text};
use iced_test::Error;
use iced_ui::Chip;
use iced_ui::fab::Fab;
use iced_ui::icon_button::{self, IconButton};
use iced_ui_tests::{DEFAULT_SIZE, build};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    FabPressed,
    ButtonPressed,
    ChipToggled,
}

#[test]
fn fab_emits_on_press_message() -> Result<(), Error> {
    let element = row![Fab::new(text("+")).on_press(Message::FabPressed)].padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    sim.click("+")?;

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::FabPressed),
        "expected FabPressed in {:?}",
        messages
    );
    Ok(())
}

#[test]
fn icon_button_emits_on_press_message() -> Result<(), Error> {
    let element = row![
        IconButton::new(text("X"))
            .variant(icon_button::Variant::Filled)
            .on_press(Message::ButtonPressed)
    ]
    .padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    sim.click("X")?;

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::ButtonPressed),
        "expected ButtonPressed in {:?}",
        messages
    );
    Ok(())
}

#[test]
fn filter_chip_emits_toggle_message() -> Result<(), Error> {
    let element = row![
        Chip::filter(text("Vegetarian"))
            .selected(false)
            .on_press(Message::ChipToggled)
    ]
    .padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    sim.click("Vegetarian")?;

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::ChipToggled),
        "expected ChipToggled in {:?}",
        messages
    );
    Ok(())
}
