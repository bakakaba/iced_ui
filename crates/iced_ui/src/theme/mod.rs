//! The `iced_ui` theme.
//!
//! [`Theme`] is the default `Theme` parameter for every widget in this
//! crate. It composes a built-in [`iced::Theme`] (the colors source)
//! with a global `spacing` and `roundness`, all of which can
//! be tweaked at runtime to restyle every `iced_ui` widget at once.
//!
//! # Token model
//!
//! Spacing and roundness are expressed as **multiplication factors** of
//! a base unit. Components ask the theme for
//! `theme.space(Space::sx(2.0))` or
//! `theme.radius(Roundness::sx(2.0))` and receive logical pixels.
//! Bumping `spacing` or `roundness` rescales every factor
//! across the UI without touching call sites — the basis for density
//! themes.
//!
//! ```
//! use iced_ui::{Space, Theme};
//!
//! let mut theme = Theme::default();
//! assert_eq!(theme.space(Space::sx(2.0)), 8.0);
//!
//! // Density rescale: bump the base, every factor follows.
//! theme.spacing = 6;
//! assert_eq!(theme.space(Space::sx(2.0)), 12.0);
//! ```
//!
//! Per-widget overrides (e.g. `Card::style`, `MenuBar::style`,
//! `Card::padding`) keep working and take precedence over the values
//! the theme provides.

mod iced_compat;
pub mod scale;

pub use scale::{FontSize, PaddingSource, Roundness, RoundnessBase, Space, SpacingBase};

use iced::theme::palette;

/// The `iced_ui` theme.
///
/// `colors` carries the underlying [`iced::Theme`] used to derive every
/// theme-aware color (palette, extended palette). `spacing` is the
/// integer step size for spacing tokens; every [`Space::sx(f)`] resolves
/// to `f * spacing`. `roundness` plays the same role for
/// [`Roundness::sx(f)`].
///
/// All three fields are public so application code can tweak them
/// directly. Construct one with [`Theme::default`] (Dark colors,
/// `spacing = 4`, `roundness = 4`) or by setting fields
/// explicitly.
///
/// [`Space::sx(f)`]: scale::Space::sx
/// [`Roundness::sx(f)`]: scale::Roundness::sx
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// The underlying built-in [`iced::Theme`] used as the source of
    /// truth for every color.
    pub colors: iced::Theme,
    /// Base unit for spacing tokens. Drives every
    /// [`Space::sx(f)`](scale::Space::sx) — `sx(2.0)` resolves to
    /// `2.0 * spacing`. Increase for a roomier UI; decrease for
    /// a denser one.
    pub spacing: u8,
    /// Base unit for roundness tokens. Drives every
    /// [`Roundness::sx(f)`](scale::Roundness::sx) — `sx(2.0)` resolves
    /// to `2.0 * roundness`.
    pub roundness: u8,
}

impl Theme {
    /// The default base unit for spacing tokens.
    pub const DEFAULT_SPACING: u8 = 4;

    /// The default base unit for roundness tokens.
    pub const DEFAULT_ROUNDNESS: u8 = 4;

    /// Returns the [`iced::theme::Palette`] of the underlying
    /// [`iced::Theme`].
    pub fn palette(&self) -> iced::theme::Palette {
        self.colors.palette()
    }

    /// Returns the [`palette::Extended`] of the underlying
    /// [`iced::Theme`].
    pub fn extended_palette(&self) -> &palette::Extended {
        self.colors.extended_palette()
    }

    /// Resolves a [`Space`] token against this theme's
    /// [`spacing`](Self::spacing).
    pub fn space(&self, space: scale::Space) -> f32 {
        space.resolve(self.spacing)
    }

    /// Resolves a [`Roundness`] token against this
    /// theme's [`roundness`](Self::roundness).
    pub fn radius(&self, roundness: scale::Roundness) -> f32 {
        roundness.resolve(self.roundness)
    }

    /// Resolves a [`PaddingSource`] into an
    /// absolute [`iced::Padding`] using this theme's
    /// [`spacing`](Self::spacing).
    pub fn padding(&self, padding: scale::PaddingSource) -> iced::Padding {
        padding.resolve(self.spacing)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            colors: iced::Theme::Dark,
            spacing: Self::DEFAULT_SPACING,
            roundness: Self::DEFAULT_ROUNDNESS,
        }
    }
}

impl From<iced::Theme> for Theme {
    fn from(colors: iced::Theme) -> Self {
        Self {
            colors,
            spacing: Self::DEFAULT_SPACING,
            roundness: Self::DEFAULT_ROUNDNESS,
        }
    }
}

impl scale::SpacingBase for Theme {
    fn spacing(&self) -> u8 {
        self.spacing
    }
}

impl scale::RoundnessBase for Theme {
    fn roundness(&self) -> u8 {
        self.roundness
    }
}
