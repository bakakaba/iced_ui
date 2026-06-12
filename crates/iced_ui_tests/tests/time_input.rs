//! Snapshot and interaction tests for the [`iced_ui::TimeInput`] widget.

use iced::mouse::{self, ScrollDelta};
use iced::{Event, Point};
use iced_test::Error;
use iced_test::simulator::click;
use iced_ui::TimeInput;
use iced_ui::chrono::NaiveTime;
use iced_ui_tests::{DEFAULT_SIZE, TALL_SIZE, assert_snapshot, build, snapshot_path, theme};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    Changed(NaiveTime),
}

fn time(h: u32, m: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, 0).unwrap()
}

/// The element under test: a `TimeInput` at (20, 20) showing 09:30.
fn element() -> iced::widget::Row<'static, Message, iced_ui::Theme> {
    iced::widget::row![TimeInput::new(time(9, 30)).on_change(Message::Changed)].padding(20)
}

/// Center of the trailing clock trigger (field is 110 wide, trigger
/// occupies the right 28px, field height ≈ 45px).
const TRIGGER: Point = Point::new(116.0, 42.0);

// The popup opens below the field (bottom ≈ 64.8 + 4 offset) at
// x = 20. The hour and minute columns sit under a 16px label row,
// spanning y 92.8–260.8 (7 items of 24px); the hour column covers
// x 28–133 and the minute column x 133–238. On open, each column is
// scrolled so the selected item (09 / 30) is vertically centered,
// i.e. drawn at y 164.8–188.8.

/// Center of the hour item two rows below the centered selection
/// (hour 11 with the column scrolled to center hour 09).
const HOUR_PLUS_TWO: Point = Point::new(80.5, 224.8);

/// Center of the minute item two rows below the centered selection
/// (minute 32 with the column scrolled to center minute 30).
const MINUTE_PLUS_TWO: Point = Point::new(185.5, 224.8);

#[test]
fn time_input_default() -> Result<(), Error> {
    assert_snapshot::<Message>("time_input_default", element(), DEFAULT_SIZE)
}

#[test]
fn time_input_open() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(snapshot_path("time_input_open"))?,
        "snapshot mismatch for time_input_open",
    );
    Ok(())
}

#[test]
fn time_input_minute_step() -> Result<(), Error> {
    let element = iced::widget::row![
        TimeInput::new(time(9, 30))
            .minute_step(15)
            .on_change(Message::Changed)
    ]
    .padding(20);

    let mut sim = build(element, TALL_SIZE);
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // The minute column offers only 00/15/30/45, which fit without
    // scrolling (no scrollbar).
    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(snapshot_path("time_input_minute_step"))?,
        "snapshot mismatch for time_input_minute_step",
    );
    Ok(())
}

#[test]
fn time_input_emits_and_stays_open_on_column_clicks() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);

    // Open the popup via the trigger.
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Pick minute 32 first: the popup must stay open so the hour can
    // still be changed afterwards.
    sim.point_at(MINUTE_PLUS_TWO);
    let _ = sim.simulate(click());

    // Then pick hour 11.
    sim.point_at(HOUR_PLUS_TWO);
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::Changed(time(9, 32))),
        "expected Changed(09:32) in {messages:?}",
    );
    assert_eq!(
        messages.last(),
        Some(&Message::Changed(time(11, 32))),
        "expected final Changed(11:32) in {messages:?}",
    );
    Ok(())
}

#[test]
fn time_input_drag_scrolls_without_selecting() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);

    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Press in the hour column and drag upward by two items (48px):
    // the content follows the cursor and releasing selects nothing.
    sim.point_at(HOUR_PLUS_TWO);
    let _ = sim.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);

    let lifted = Point::new(HOUR_PLUS_TWO.x, HOUR_PLUS_TWO.y - 48.0);
    sim.point_at(lifted);
    let _ = sim.simulate([Event::Mouse(mouse::Event::CursorMoved { position: lifted })]);
    let _ = sim.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);

    // After scrolling down two items, hour 11 sits where hour 09 was
    // centered; click it.
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec![Message::Changed(time(11, 30))],
        "the drag must not select; only the follow-up click may emit",
    );
    Ok(())
}

#[test]
fn time_input_wheel_scrolls_hour_column() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);

    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Scroll the hour column up by one line (one 24px item): every
    // visible hour shifts down by one row.
    sim.point_at(Point::new(80.5, 176.8));
    let _ = sim.simulate([Event::Mouse(mouse::Event::WheelScrolled {
        delta: ScrollDelta::Lines { x: 0.0, y: 1.0 },
    })]);

    // The position two rows below center now holds hour 10.
    sim.point_at(HOUR_PLUS_TWO);
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::Changed(time(10, 30))),
        "expected Changed(10:30) in {messages:?}",
    );
    Ok(())
}
