//! Interaction tests demonstrating non-snapshot behavioral testing.
//!
//! These tests exercise widgets via `Simulator` clicks and key taps,
//! and assert on the resulting message stream rather than the rendered
//! pixels. They guarantee that user input is correctly translated into
//! application messages without depending on a golden image.

use iced::widget::{row, text};
use iced_test::Error;
use iced_ui::Chip;
use iced_ui::checkbox::Checkbox;
use iced_ui::fab::Fab;
use iced_ui::icon_button::{self, IconButton};
use iced_ui_tests::{DEFAULT_SIZE, build};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    FabPressed,
    ButtonPressed,
    ChipToggled,
    BoolToggled(bool),
    OptToggled(Option<bool>),
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

#[test]
fn binary_checkbox_toggles_bool() -> Result<(), Error> {
    // A `false` checkbox clicked by the user reports `true`.
    let element = row![
        Checkbox::new(false)
            .label(text("Accept"))
            .on_toggle(Message::BoolToggled)
    ]
    .padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    sim.click("Accept")?;

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::BoolToggled(true)),
        "expected BoolToggled(true) in {:?}",
        messages
    );
    Ok(())
}

#[test]
fn tristate_checkbox_can_cycle_into_indeterminate() -> Result<(), Error> {
    // An `Option<bool>` checkbox in the `Some(false)` value cycles to
    // the indeterminate (`None`) value when clicked, proving the user
    // can reach indeterminate.
    let element = row![
        Checkbox::new(Some(false))
            .label(text("Any"))
            .on_toggle(Message::OptToggled)
    ]
    .padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    sim.click("Any")?;

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::OptToggled(None)),
        "expected OptToggled(None) in {:?}",
        messages
    );
    Ok(())
}
