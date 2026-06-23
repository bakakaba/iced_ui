//! Snapshot tests for the [`iced_ui::Progress`] widget.
//!
//! Each test renders the widget using its default configuration (per
//! the project's "Demo showcases defaults" rule) and compares the
//! result against a golden PNG reference image in `tests/snapshots/`.
//!
//! The indeterminate mode animates over time, but the first rendered
//! frame (before any `RedrawRequested` is observed) is pinned to phase
//! 0, so these snapshots are deterministic.

use iced::Length;
use iced::widget::{container, row};
use iced_test::Error;
use iced_ui::Progress;
use iced_ui::progress::{Color, Dock};
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {}

/// Constrain the full-width bar to a fixed width so snapshots are
/// stable and the bar doesn't span the whole canvas.
fn bar(
    element: impl Into<iced_ui_tests::Element<'static, Message>>,
) -> iced_ui_tests::Element<'static, Message> {
    row![container(element).width(Length::Fixed(240.0))]
        .padding(20)
        .into()
}

#[test]
fn progress_determinate_default() -> Result<(), Error> {
    assert_snapshot::<Message>(
        "progress_determinate_default",
        bar(Progress::determinate(0.6)),
        DEFAULT_SIZE,
    )
}

#[test]
fn progress_determinate_empty() -> Result<(), Error> {
    assert_snapshot::<Message>(
        "progress_determinate_empty",
        bar(Progress::determinate(0.0)),
        DEFAULT_SIZE,
    )
}

#[test]
fn progress_determinate_full() -> Result<(), Error> {
    assert_snapshot::<Message>(
        "progress_determinate_full",
        bar(Progress::determinate(1.0)),
        DEFAULT_SIZE,
    )
}

#[test]
fn progress_indeterminate_default() -> Result<(), Error> {
    assert_snapshot::<Message>(
        "progress_indeterminate_default",
        bar(Progress::indeterminate()),
        DEFAULT_SIZE,
    )
}

#[test]
fn progress_determinate_success() -> Result<(), Error> {
    assert_snapshot::<Message>(
        "progress_determinate_success",
        bar(Progress::determinate(0.6).color(Color::Success)),
        DEFAULT_SIZE,
    )
}

#[test]
fn progress_determinate_danger() -> Result<(), Error> {
    assert_snapshot::<Message>(
        "progress_determinate_danger",
        bar(Progress::determinate(0.6).color(Color::Danger)),
        DEFAULT_SIZE,
    )
}

#[test]
fn progress_docked_top() -> Result<(), Error> {
    // Docked to the top: flat top corners, elevated by default.
    assert_snapshot::<Message>(
        "progress_docked_top",
        bar(Progress::determinate(0.6).dock(Dock::Top)),
        DEFAULT_SIZE,
    )
}

#[test]
fn progress_docked_bottom() -> Result<(), Error> {
    // Docked to the bottom: flat bottom corners, elevated by default.
    assert_snapshot::<Message>(
        "progress_docked_bottom",
        bar(Progress::determinate(0.6).dock(Dock::Bottom)),
        DEFAULT_SIZE,
    )
}

#[test]
fn progress_docked_no_elevation() -> Result<(), Error> {
    // Docking with elevation explicitly suppressed: flat corners, no
    // shadow.
    assert_snapshot::<Message>(
        "progress_docked_no_elevation",
        bar(Progress::determinate(0.6).dock(Dock::Top).elevated(false)),
        DEFAULT_SIZE,
    )
}
