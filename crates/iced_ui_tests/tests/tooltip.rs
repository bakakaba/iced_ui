//! Snapshot tests for the [`Tooltip`](iced_ui::Tooltip) widget.
//!
//! The bubble is only rendered while the trigger is hovered (in an
//! overlay), so the default snapshot captures just the trigger, and the
//! remaining tests drive the cursor over the trigger to capture the
//! revealed bubble and its directional caret at each position.

use iced::widget::{container, row, text};
use iced_test::Error;
use iced_ui::Tooltip;
use iced_ui::tooltip::Position;
use iced_ui_tests::{DEFAULT_SIZE, TALL_SIZE, assert_snapshot, build, theme};

#[derive(Debug, Clone)]
enum Message {}

#[test]
fn tooltip_default() -> Result<(), Error> {
    let element = row![Tooltip::new(
        container(text("Hover me")).padding([8, 16]),
        text("Tooltip content"),
        Position::Top,
    )]
    .padding(20);

    assert_snapshot::<Message>("tooltip_default", element, DEFAULT_SIZE)
}

#[test]
fn tooltip_revealed_on_hover() -> Result<(), Error> {
    let element = row![Tooltip::new(
        container(text("Hover me")).padding([8, 16]),
        text("Tooltip content"),
        Position::Bottom,
    )]
    .padding(20);

    let mut sim = build::<Message>(element, DEFAULT_SIZE);
    // Lay the widget out once, then move the cursor over the trigger so
    // the tooltip's hover state opens the overlay before snapshotting.
    let _ = sim.snapshot(&theme());
    sim.point_at(iced::Point::new(60.0, 40.0));
    let _ = sim.snapshot(&theme());

    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(iced_ui_tests::snapshot_path("tooltip_revealed_on_hover"))?,
        "tooltip_revealed_on_hover snapshot mismatch",
    );
    Ok(())
}

#[test]
fn tooltip_caret_top() -> Result<(), Error> {
    // Position::Top places the bubble above the trigger, so the caret
    // sits on the bubble's bottom edge pointing down at it. The trigger
    // is pushed down the tall canvas so there is room above.
    let element = container(row![Tooltip::new(
        container(text("Hover me")).padding([8, 16]),
        text("Tooltip content"),
        Position::Top,
    )])
    .padding(20)
    .center_y(iced::Length::Fill);

    let mut sim = build::<Message>(element, TALL_SIZE);
    let _ = sim.snapshot(&theme());
    sim.point_at(iced::Point::new(70.0, 240.0));
    let _ = sim.snapshot(&theme());

    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(iced_ui_tests::snapshot_path("tooltip_caret_top"))?,
        "tooltip_caret_top snapshot mismatch",
    );
    Ok(())
}

#[test]
fn tooltip_caret_left() -> Result<(), Error> {
    // Position::Left places the bubble to the left of the trigger, so
    // the caret sits on the bubble's right edge pointing right at it.
    // The trigger is centered so there is room on the left.
    let element = container(Tooltip::new(
        container(text("Hover me")).padding([8, 16]),
        text("Tip"),
        Position::Left,
    ))
    .padding(20)
    .center_x(iced::Length::Fill)
    .center_y(iced::Length::Fill);

    let mut sim = build::<Message>(element, DEFAULT_SIZE);
    let _ = sim.snapshot(&theme());
    sim.point_at(iced::Point::new(180.0, 120.0));
    let _ = sim.snapshot(&theme());

    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(iced_ui_tests::snapshot_path("tooltip_caret_left"))?,
        "tooltip_caret_left snapshot mismatch",
    );
    Ok(())
}

#[test]
fn tooltip_caret_right() -> Result<(), Error> {
    // Position::Right places the bubble to the right of the trigger, so
    // the caret sits on the bubble's left edge pointing left at it.
    let element = container(Tooltip::new(
        container(text("Hover me")).padding([8, 16]),
        text("Tip"),
        Position::Right,
    ))
    .padding(20)
    .center_x(iced::Length::Fill)
    .center_y(iced::Length::Fill);

    let mut sim = build::<Message>(element, DEFAULT_SIZE);
    let _ = sim.snapshot(&theme());
    sim.point_at(iced::Point::new(140.0, 120.0));
    let _ = sim.snapshot(&theme());

    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(iced_ui_tests::snapshot_path("tooltip_caret_right"))?,
        "tooltip_caret_right snapshot mismatch",
    );
    Ok(())
}
