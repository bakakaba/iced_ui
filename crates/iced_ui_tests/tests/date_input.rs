//! Snapshot and interaction tests for the [`iced_ui::DateInput`] widget.

use iced::keyboard::key::Named;
use iced::{Event, Point, mouse};
use iced_test::Error;
use iced_test::simulator::click;
use iced_ui::DateInput;
use iced_ui::chrono::NaiveDate;
use iced_ui_tests::{DEFAULT_SIZE, TALL_SIZE, assert_snapshot, build, snapshot_path, theme};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    Changed(NaiveDate),
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// The element under test: a `DateInput` at (20, 20) showing
/// 2026-06-12 (June 2026 starts on a Monday, so the popup's first
/// grid cell is June 1). "Today" is pinned to 2026-06-03 so the
/// popup snapshots don't depend on the host's real date.
fn element() -> iced::widget::Row<'static, Message, iced_ui::Theme> {
    iced::widget::row![
        DateInput::new(date(2026, 6, 12))
            .today(date(2026, 6, 3))
            .on_change(Message::Changed)
    ]
    .padding(20)
}

/// Center of the trailing calendar trigger (field is 150 wide,
/// trigger occupies the right 28px, field height ≈ 45px).
const TRIGGER: Point = Point::new(156.0, 42.0);

#[test]
fn date_input_default() -> Result<(), Error> {
    assert_snapshot::<Message>("date_input_default", element(), DEFAULT_SIZE)
}

#[test]
fn date_input_open() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(snapshot_path("date_input_open"))?,
        "snapshot mismatch for date_input_open",
    );
    Ok(())
}

#[test]
fn date_input_emits_on_day_click() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);

    // Open the popup via the trigger.
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Click the first day cell (June 1, 2026). The popup opens below
    // the field (bottom ≈ 64.8 + 4 offset): first cell center is at
    // popup origin + padding + (15, header 28 + weekdays 20 + 13).
    sim.point_at(Point::new(43.0, 137.8));
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::Changed(date(2026, 6, 1))),
        "expected Changed(2026-06-01) in {messages:?}",
    );
    Ok(())
}

/// Center of the previous-month chevron of the popup header
/// (chevrons are 28 wide; the header row spans y 76.8–104.8).
const PREV_CHEVRON: Point = Point::new(42.0, 90.8);

/// Center of the next-month chevron of the popup header.
const NEXT_CHEVRON: Point = Point::new(224.0, 90.8);

/// Center of the clickable month segment of the popup header (title
/// spans x 56–210; the month takes the left 58%).
const MONTH_TITLE: Point = Point::new(100.0, 90.8);

/// Center of the clickable year segment of the popup header.
const YEAR_TITLE: Point = Point::new(177.0, 90.8);

#[test]
fn date_input_chevron_hover() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Hover the previous-month chevron: it gets the same pill
    // highlight as the title segments.
    sim.point_at(PREV_CHEVRON);

    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(snapshot_path("date_input_chevron_hover"))?,
        "snapshot mismatch for date_input_chevron_hover",
    );
    Ok(())
}

#[test]
fn date_input_months_view() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Click the month in the header: the body switches to the
    // 3 × 4 month list.
    sim.point_at(MONTH_TITLE);
    let _ = sim.simulate(click());

    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(snapshot_path("date_input_months_view"))?,
        "snapshot mismatch for date_input_months_view",
    );
    Ok(())
}

#[test]
fn date_input_years_view() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Click the year in the header: the body switches to the
    // 4 × 5 year page (2020 – 2039).
    sim.point_at(YEAR_TITLE);
    let _ = sim.simulate(click());

    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(snapshot_path("date_input_years_view"))?,
        "snapshot mismatch for date_input_years_view",
    );
    Ok(())
}

#[test]
fn date_input_month_list_changes_view_month() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Header month → month list.
    sim.point_at(MONTH_TITLE);
    let _ = sim.simulate(click());

    // Click "Mar" (index 2; body cells are 70 × 44 starting at
    // (28, 104.8)): back to the day view showing March 2026.
    sim.point_at(Point::new(203.0, 126.8));
    let _ = sim.simulate(click());

    // March 2026 starts on a Sunday, so the grid starts Monday
    // Feb 23 and March 12 is cell 17 (row 2, column 3).
    sim.point_at(Point::new(133.0, 189.8));
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::Changed(date(2026, 3, 12))),
        "expected Changed(2026-03-12) in {messages:?}",
    );
    Ok(())
}

#[test]
fn date_input_year_list_changes_view_year() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);
    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Header year → year list.
    sim.point_at(YEAR_TITLE);
    let _ = sim.simulate(click());

    // Click 2031 (index 11 on the 2020 – 2039 page; body cells are
    // 52.5 × 35.2 starting at (28, 104.8)): back to the day view
    // showing June 2031.
    sim.point_at(Point::new(211.75, 192.8));
    let _ = sim.simulate(click());

    // June 2031 starts on a Sunday, so the grid starts Monday
    // May 26 and June 12 is cell 17 (row 2, column 3).
    sim.point_at(Point::new(133.0, 189.8));
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::Changed(date(2031, 6, 12))),
        "expected Changed(2031-06-12) in {messages:?}",
    );
    Ok(())
}

#[test]
fn date_input_drag_selects_release_cell() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);

    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Press on June 1, drag one row down, and release on June 8: the
    // day cell under the release position wins.
    sim.point_at(Point::new(43.0, 137.8));
    let _ = sim.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);

    let other_cell = Point::new(43.0, 163.8);
    sim.point_at(other_cell);
    let _ = sim.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: other_cell,
    })]);
    let _ = sim.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::Changed(date(2026, 6, 8))),
        "expected Changed(2026-06-08) in {messages:?}",
    );
    Ok(())
}

#[test]
fn date_input_drag_from_cell_to_chevron_selects_nothing() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);

    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());

    // Press on June 1 but release over the next-month chevron: cells
    // and header buttons don't mix, so nothing activates (no message
    // and no month navigation).
    sim.point_at(Point::new(43.0, 137.8));
    let _ = sim.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);

    sim.point_at(NEXT_CHEVRON);
    let _ = sim.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: NEXT_CHEVRON,
    })]);
    let _ = sim.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);

    // The popup must still show June 2026: clicking the first grid
    // cell selects June 1 (had the chevron fired, the view would show
    // July and the first cell would be June 29).
    sim.point_at(Point::new(43.0, 137.8));
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec![Message::Changed(date(2026, 6, 1))],
        "expected only Changed(2026-06-01), got {messages:?}",
    );
    Ok(())
}

#[test]
fn date_input_emits_on_typed_date() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);

    // Click the text area (right of the text, left of the trigger) to
    // focus the inner text input with the cursor at the end.
    sim.point_at(Point::new(125.0, 42.0));
    let _ = sim.simulate(click());

    // Erase "2026-06-12" and type a new date.
    for _ in 0..10 {
        let _ = sim.tap_key(Named::Backspace);
    }
    let _ = sim.typewrite("2027-01-15");

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert_eq!(
        messages.last(),
        Some(&Message::Changed(date(2027, 1, 15))),
        "expected final Changed(2027-01-15) in {messages:?}",
    );
    Ok(())
}

#[test]
fn date_input_escape_closes_popup() -> Result<(), Error> {
    let mut sim = build(element(), TALL_SIZE);

    sim.point_at(TRIGGER);
    let _ = sim.simulate(click());
    let _ = sim.tap_key(Named::Escape);

    // After Escape the popup must be gone: the result should match
    // the default (closed) appearance at the same canvas size.
    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(snapshot_path("date_input_closed_after_escape"))?,
        "snapshot mismatch for date_input_closed_after_escape",
    );
    Ok(())
}
