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
    ChipRemoved,
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
fn chip_emits_toggle_message() -> Result<(), Error> {
    use iced_test::simulator::click;

    let element = row![Chip::new("Vegetarian").on_toggle(Message::ChipToggled)].padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    // The chip renders its label directly (not as a `text` widget), so
    // target the chip body by a point inside its bounds rather than by
    // text selector.
    sim.point_at(iced::Point::new(50.0, 35.0));
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::ChipToggled),
        "expected ChipToggled in {:?}",
        messages
    );
    Ok(())
}

#[test]
fn chip_readonly_emits_nothing() -> Result<(), Error> {
    use iced_test::simulator::click;

    // No handler => static token; clicking must not emit anything.
    let element = row![Chip::<Message>::new("Read only")].padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    sim.point_at(iced::Point::new(50.0, 35.0));
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.is_empty(),
        "expected no messages from a read-only chip, got {:?}",
        messages
    );
    Ok(())
}

#[test]
fn chip_remove_button_emits_remove() -> Result<(), Error> {
    use iced_test::simulator::click;

    let element = row![
        Chip::new("John Doe")
            .on_toggle(Message::ChipToggled)
            .on_remove(Message::ChipRemoved)
    ]
    .padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    // Snapshot once so the chip lays out (caching its geometry) and the
    // remove button's bounds are established near the trailing edge.
    let _ = sim.snapshot(&iced_ui_tests::theme());
    // Click the remove circle near the trailing edge of the pill. The
    // pill is ~91px wide starting at x=20; the circle is centered at
    // roughly x=95, y=32.
    sim.point_at(iced::Point::new(95.0, 32.0));
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::ChipRemoved),
        "expected ChipRemoved in {:?}",
        messages
    );
    assert!(
        !messages.contains(&Message::ChipToggled),
        "clicking the remove button must not also toggle: {:?}",
        messages
    );
    Ok(())
}

#[test]
fn chip_remove_only_body_click_does_nothing() -> Result<(), Error> {
    use iced_test::simulator::click;

    // A chip with only `on_remove` is not body-interactive: clicking the
    // label region must not emit anything (only the x button removes).
    let element = row![Chip::new("John Doe").on_remove(Message::ChipRemoved)].padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    let _ = sim.snapshot(&iced_ui_tests::theme());
    // Click the label region (left side), away from the trailing x.
    sim.point_at(iced::Point::new(45.0, 34.0));
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.is_empty(),
        "body click on a remove-only chip must emit nothing, got {:?}",
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
    // An `Option<bool>` checkbox with `.indeterminate(true)` in the
    // `Some(false)` value cycles to the indeterminate (`None`) value
    // when clicked, proving the user can reach indeterminate in the
    // cyclable mode.
    let element = row![
        Checkbox::new(Some(false))
            .label(text("Any"))
            .indeterminate(true)
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

#[test]
fn readonly_indeterminate_click_selects_checked() -> Result<(), Error> {
    // The default (read-only) tri-state checkbox starts indeterminate
    // (`None`); a click selects (`Some(true)`) rather than cycling.
    let element = row![
        Checkbox::new(None)
            .label(text("Select all"))
            .on_toggle(Message::OptToggled)
    ]
    .padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    sim.click("Select all")?;

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::OptToggled(Some(true))),
        "expected OptToggled(Some(true)) in {:?}",
        messages
    );
    Ok(())
}

#[test]
fn readonly_indeterminate_never_returns_to_indeterminate() -> Result<(), Error> {
    // In the default (read-only) mode, a click from `Some(false)` moves
    // to `Some(true)` rather than to the indeterminate (`None`) value.
    let element = row![
        Checkbox::new(Some(false))
            .label(text("Select all"))
            .on_toggle(Message::OptToggled)
    ]
    .padding(20);

    let mut sim = build(element, DEFAULT_SIZE);
    sim.click("Select all")?;

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::OptToggled(Some(true))),
        "expected OptToggled(Some(true)) in {:?}",
        messages
    );
    assert!(
        !messages.contains(&Message::OptToggled(None)),
        "read-only mode must never emit OptToggled(None), got {:?}",
        messages
    );
    Ok(())
}
