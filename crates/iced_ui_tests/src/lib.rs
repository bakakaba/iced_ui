//! Shared test utilities for `iced_ui` widget snapshot tests.
//!
//! This crate is internal and never published. It exposes a small
//! amount of shared infrastructure used by the integration tests in
//! the `tests/` directory.
//!
//! ## Why a separate crate?
//!
//! These tests could live in `crates/iced_ui/tests/` with
//! `iced_test` as a dev-dependency. They are split into their own
//! workspace member on purpose:
//!
//! - **Publish hygiene.** `iced_ui` is the only crate published to
//!   crates.io and its dependency surface must stay minimal. Keeping
//!   the tests here keeps the snapshot PNGs under `tests/snapshots/`,
//!   the bundled fonts under `fonts/`, and the heavy `iced_test`
//!   dev-dependency (which drags in renderer backends) out of the
//!   published package and its `cargo publish` verification build.
//! - **Shared harness library.** Each `tests/*.rs` integration test
//!   is compiled as its own crate, so test files cannot share code
//!   with each other directly. The helpers in this `lib.rs`
//!   ([`assert_snapshot`], [`build`], [`theme`], the font-system
//!   setup) need a library to live in — and placing them inside
//!   `iced_ui` would mean shipping test-only API (or hiding it
//!   behind a feature flag).
//! - **Independent feature control.** This crate enables `iced`
//!   features (`image`, `svg`) and the `iced_ui/lucide` feature that
//!   the library does not enable by default, exercising optional
//!   surface without forcing it on downstream users. Pinning
//!   `lucide` here also keeps snapshot output identical between
//!   `cargo test -p iced_ui_tests` and `cargo test --workspace`,
//!   where feature unification with the demo crate would otherwise
//!   toggle it on only for workspace builds.
//! - **Build hygiene.** Churn in the test harness or snapshot
//!   references never invalidates `iced_ui` itself, so library
//!   builds and downstream consumers are unaffected.
//!
//! The testing approach is:
//!
//! 1. Wrap each widget in a deterministic [`iced::Element`] using its
//!    default configuration.
//! 2. Drive it through [`iced_test::Simulator`] which renders to a
//!    headless backend (CPU `tiny-skia` when `ICED_TEST_BACKEND` is
//!    set, otherwise `wgpu` if a GPU is available).
//! 3. Compare the resulting [`iced_test::simulator::Snapshot`] against a golden
//!    PNG committed under `tests/snapshots/`.
//!
//! Snapshots are namespaced per backend by `iced_test`; the file name
//! ends in `-tiny-skia.png` or `-wgpu.png` automatically, so reference
//! images for both backends can coexist.
//!
//! ## Bundled fonts and font resolution
//!
//! `iced_test` only ships `FiraSans-Regular.ttf` and remaps the
//! renderer's default font to the named family `"Fira Sans"`. That
//! makes widgets which render text via `renderer.default_font()`
//! deterministic, because the lookup goes by family name and matches
//! the bundled face exactly.
//!
//! However, widgets like [`iced_ui::Text`] explicitly request a bold
//! weight using the generic [`iced::Font`]`{ family: SansSerif,
//! weight: Bold, .. }` descriptor. When `cosmic_text` resolves
//! `Family::SansSerif` it consults a hardcoded alias that defaults to
//! `"Open Sans"` — which is not bundled — so the font database falls
//! back to whatever sans-serif bold face the host happens to have
//! installed. That's non-deterministic across CI runners and dev
//! machines.
//!
//! To make those snapshots reproducible, this crate:
//!
//! 1. Bundles `FiraSans-Bold.ttf` (SIL Open Font License) under
//!    `fonts/` and loads it into the global font system.
//! 2. Overrides the sans-serif family alias to `"Fira Sans"` so any
//!    generic `Family::SansSerif` lookup (regardless of weight)
//!    resolves to the bundled Fira Sans family.
//!
//! Both steps run exactly once per test process, behind a
//! [`std::sync::Once`], before the first [`Simulator`] is built.

use std::path::PathBuf;
use std::sync::Once;

use iced::{Element as IcedElement, Settings, Theme as IcedTheme};
use iced_test::{Error, Simulator};
use iced_ui::Theme;

/// Convenience alias for an `iced::Element` already typed against the
/// `iced_ui::Theme` used in tests.
pub type Element<'a, Message, Theme = iced_ui::Theme> = IcedElement<'a, Message, Theme>;

/// Bold variant of Fira Sans, bundled to make sans-serif bold
/// rendering deterministic across environments.
const FIRA_SANS_BOLD: &[u8] = include_bytes!("../fonts/FiraSans-Bold.ttf");

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

/// Run-once initialization of the global `cosmic_text` font system
/// used by every `iced_test` simulator in this process.
///
/// This function:
///
/// - Loads the bundled `FiraSans-Bold.ttf` so a sans-serif bold face
///   is available in the font database.
/// - Sets the sans-serif family alias to `"Fira Sans"` so generic
///   `Family::SansSerif` requests resolve to the bundled family
///   (matching both the regular face shipped by `iced_test` and the
///   bold face bundled here).
/// - Loads the Lucide icon font so widgets that render icon glyphs
///   via [`iced_ui::icons::FONT`] (e.g. the snackbar's severity and
///   dismiss icons) resolve to real glyphs instead of a host
///   fallback. This mirrors what the demo app does via
///   `iced::application(...).font(FONT_BYTES)`.
fn init_font_system() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let mut system = iced::advanced::graphics::text::font_system()
            .write()
            .expect("acquire font_system write lock");

        // Make `Family::SansSerif` resolve to "Fira Sans" so that
        // widget code requesting a generic sans-serif family (in any
        // weight) hits the bundled fonts rather than whatever
        // sans-serif face the host has installed.
        system.raw().db_mut().set_sans_serif_family("Fira Sans");

        // Load the bundled bold face so requests for Fira Sans Bold
        // succeed against an in-memory, deterministic file.
        system.load_font(std::borrow::Cow::Borrowed(FIRA_SANS_BOLD));

        // Load the Lucide icon font so icon glyphs render
        // deterministically (the demo app loads this at startup).
        system.load_font(std::borrow::Cow::Borrowed(iced_ui::icons::FONT_BYTES));
    });
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
    init_font_system();

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
