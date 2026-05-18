//! Shared test utilities for `iced_ui` widget snapshot tests.
//!
//! This crate is internal and never published. It exposes a small
//! amount of shared infrastructure used by the integration tests in
//! the `tests/` directory.
//!
//! The testing approach is:
//!
//! 1. Wrap each widget in a deterministic [`iced::Element`] using its
//!    default configuration.
//! 2. Drive it through [`iced_test::Simulator`] which renders to a
//!    headless backend (CPU `tiny-skia` when `ICED_TEST_BACKEND` is
//!    set, otherwise `wgpu` if a GPU is available).
//! 3. Compare the resulting [`iced_test::Snapshot`] against a golden
//!    PNG committed under `tests/snapshots/`.
//!
//! Snapshots are namespaced per backend by `iced_test`; the file name
//! ends in `-tiny-skia.png` or `-wgpu.png` automatically, so reference
//! images for both backends can coexist.

use std::path::PathBuf;

use iced::{Element as IcedElement, Settings, Theme as IcedTheme};
use iced_test::{Error, Simulator};
use iced_ui::Theme;

/// Convenience alias for an `iced::Element` already typed against the
/// `iced_ui::Theme` used in tests.
pub type Element<'a, Message, Theme = iced_ui::Theme> = IcedElement<'a, Message, Theme>;

/// Default canvas size used by widget snapshots.
pub const DEFAULT_SIZE: (u32, u32) = (320, 240);

/// A wide canvas, suitable for top app bars / navigation rails.
pub const WIDE_SIZE: (u32, u32) = (640, 120);

/// A tall canvas, suitable for navigation drawers.
pub const TALL_SIZE: (u32, u32) = (320, 480);

/// Returns the `iced_ui` default `Theme`, derived from the iced
/// `Light` palette so snapshots are deterministic regardless of host
/// preferences.
pub fn theme() -> Theme {
    Theme::from(IcedTheme::Light)
}

/// Build a `Simulator` for the supplied widget element with the given
/// canvas size.
pub fn build<'a, Message>(
    element: impl Into<IcedElement<'a, Message, Theme>>,
    (width, height): (u32, u32),
) -> Simulator<'a, Message, Theme>
where
    Message: 'a,
{
    Simulator::with_size(
        Settings::default(),
        iced::Size::new(width as f32, height as f32),
        element,
    )
}

/// Resolve the path to a snapshot reference image, without the
/// backend suffix or `.png` extension that `iced_test` will append.
///
/// `iced_test` writes/reads `<name>-<renderer>.png`, so passing
/// `snapshot_path("badge_default")` produces e.g.
/// `tests/snapshots/badge_default-tiny-skia.png`.
pub fn snapshot_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("snapshots");
    path.push(name);
    path
}

/// Convenience: render the supplied element with the default theme
/// and assert it matches the named snapshot.
///
/// Creates the reference image on first run.
pub fn assert_snapshot<'a, Message>(
    name: &str,
    element: impl Into<IcedElement<'a, Message, Theme>>,
    size: (u32, u32),
) -> Result<(), Error>
where
    Message: 'a,
{
    let mut sim = build(element, size);
    let snapshot = sim.snapshot(&theme())?;
    assert!(
        snapshot.matches_image(snapshot_path(name))?,
        "snapshot mismatch for {name}",
    );
    Ok(())
}
