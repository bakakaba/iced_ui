//! Snapshot and interaction tests for the [`iced_ui::DateTimeInput`]
//! widget.

use iced::Point;
use iced_test::Error;
use iced_test::simulator::click;
use iced_ui::DateTimeInput;
use iced_ui::chrono::{NaiveDate, NaiveDateTime};
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot, build, snapshot_path, theme};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    Changed(NaiveDateTime),
}

fn datetime(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(y, mo, d)
        .unwrap()
        .and_hms_opt(h, mi, 0)
        .unwrap()
}

/// A canvas tall enough for the combined calendar + time popup
/// (popup is ~412px tall and opens below the ~45px field).
const POPUP_SIZE: (u32, u32) = (340, 560);

/// The element under test: a `DateTimeInput` at (20, 20) showing
/// 2026-06-12 09:30. "Today" is pinned to 2026-06-03 so the popup
/// snapshots don't depend on the host's real date.
fn element() -> iced::widget::Row<'static, Message, iced_ui::Theme> {
    iced::widget::row![
        DateTimeInput::new(datetime(2026, 6, 12, 9, 30))
            .today(NaiveDate::from_ymd_opt(2026, 6, 3).unwrap())
            .on_change(Message::Changed)
    ]
    .padding(20)
}

/// Center of the trailing calendar trigger (field is 210 wide,
/// trigger occupies the right 28px, field height ≈ 45px).
const TRIGGER: Point = Point::new(216.0, 42.0);

#[test]
fn datetime_input_default() -> Result<(), Error> {
    assert_snapshot::<Message>("datetime_input_default", element(), DEFAULT_SIZE)
}

#[test]
fn datetime_input_open() -> Result<(), Error> {
    let mut sim = build(element(), POPUP_SIZE);
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(snapshot_path("datetime_input_open"))?,
        "snapshot mismatch for datetime_input_open",
    );
    Ok(())
}

#[test]
fn datetime_input_emits_on_day_click_and_stays_open() -> Result<(), Error> {
    let mut sim = build(element(), POPUP_SIZE);

    // Open the popup via the trigger.
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Click the first day cell (June 1, 2026): popup opens below the
    // field (bottom ≈ 64.8 + 4 offset); first cell center is at popup
    // origin + padding + (15, header 28 + weekdays 20 + 13).
    sim.point_at(Point::new(43.0, 137.8));
    let _ = sim.simulate(click());

    // The popup stays open for date-time inputs; pick a minute too.
    // The time columns start at popup origin + padding + calendar 204
    // + gap 8 + label 16 (y = 304.8) and the minute column spans
    // x 133–238. On open the column is scrolled to center minute 30
    // (drawn at y 376.8–400.8), so two rows below center (y ≈ 436.8)
    // holds minute 32.
    sim.point_at(Point::new(185.5, 436.8));
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::Changed(datetime(2026, 6, 1, 9, 30))),
        "expected Changed(2026-06-01 09:30) in {messages:?}",
    );
    assert_eq!(
        messages.last(),
        Some(&Message::Changed(datetime(2026, 6, 1, 9, 32))),
        "expected final Changed(2026-06-01 09:32) in {messages:?}",
    );
    Ok(())
}
