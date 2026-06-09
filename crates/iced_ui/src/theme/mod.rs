//! The `iced_ui` theme.
//!
//! [`Theme`] is the default `Theme` parameter for every widget in this
//! crate. It composes a built-in [`iced::Theme`] (the colors source)
//! with a global `spacing`, `roundness`, and `text_size`, all of
//! which can be tweaked at runtime to restyle every `iced_ui` widget
//! at once.
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
//! Text sizes are absolute pixel values. Components call
//! `theme.text(factor)` where `factor` is a multiplier of the base
//! text size, and receive an `iced::Pixels` value. For example,
//! `theme.text(0.875)` at the default 16px base returns
//! `Pixels(14.0)`.
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
pub mod tokens;

pub use scale::{
    FontSize, FontSizeBase, PaddingSource, Roundness, RoundnessBase, Space, SpacingBase,
};
pub use tokens::{Information, Paper};

use iced::theme::palette;

/// The `iced_ui` theme.
///
/// `colors` carries the underlying [`iced::Theme`] used to derive every
/// theme-aware color (palette, extended palette). `spacing` is the
/// integer step size for spacing tokens; every [`Space::sx(f)`] resolves
/// to `f * spacing`. `roundness` plays the same role for
/// [`Roundness::sx(f)`]. `text_size` is the base text size in logical
/// pixels from which all widget font sizes are derived.
///
/// All fields are public so application code can tweak them
/// directly. Construct one with [`Theme::default`] (Dark colors,
/// `spacing = 4`, `roundness = 4`, `text_size = 16.0`) or by
/// setting fields explicitly.
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
    /// Base text size in logical pixels. All widget font sizes are
    /// derived as fractions or multiples of this value via
    /// [`Theme::text(factor)`](Self::text). Defaults to iced's
    /// built-in default text size (16.0).
    pub text_size: f32,
    /// Surface colors that contrast the background. Used for cards,
    /// sheets, and other surfaces that float above the base
    /// background.
    pub paper: Paper,
    /// Informational color group (typically cyan). Used for
    /// informational badges, banners, and highlights.
    pub information: Information,
}

impl Theme {
    /// The default base unit for spacing tokens.
    pub const DEFAULT_SPACING: u8 = 8;

    /// The default base unit for roundness tokens.
    pub const DEFAULT_ROUNDNESS: u8 = 8;

    /// The default base text size — matches iced's built-in
    /// [`Settings::default_text_size`](iced::Settings::default_text_size).
    pub const DEFAULT_TEXT_SIZE: f32 = 16.0;

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

    /// Returns a text size as a multiple of this theme's
    /// [`text_size`](Self::text_size).
    ///
    /// `factor` is a multiplier: `1.0` returns the base text size,
    /// `0.875` returns 87.5% of it (14px at default), `1.375` returns
    /// 137.5% (22px at default), and so on.
    ///
    /// # Example
    ///
    /// ```
    /// use iced_ui::Theme;
    ///
    /// let theme = Theme::default();
    /// assert_eq!(theme.text(1.0).0, 16.0);
    /// assert_eq!(theme.text(0.875).0, 14.0);
    /// ```
    pub fn text(&self, factor: f32) -> iced::Pixels {
        iced::Pixels(self.text_size * factor)
    }

    /// Regenerates [`paper`](Self::paper) from the current
    /// [`colors`](Self::colors) palette.
    pub fn refresh_paper(&mut self) {
        let p = self.colors.palette();
        self.paper = Paper::generate(p.background, p.text);
    }

    /// Regenerates [`information`](Self::information) from the given
    /// base color and the current [`colors`](Self::colors) palette.
    pub fn refresh_information(&mut self, base: iced::Color) {
        let p = self.colors.palette();
        self.information = Information::generate(base, p.background, p.text);
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from(iced::Theme::Dark)
    }
}

impl From<iced::Theme> for Theme {
    fn from(colors: iced::Theme) -> Self {
        let p = colors.palette();
        let info_base = Information::default_base(p.background);
        Self {
            paper: Paper::generate(p.background, p.text),
            information: Information::generate(info_base, p.background, p.text),
            colors,
            spacing: Self::DEFAULT_SPACING,
            roundness: Self::DEFAULT_ROUNDNESS,
            text_size: Self::DEFAULT_TEXT_SIZE,
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

impl scale::FontSizeBase for Theme {
    fn text_size(&self) -> f32 {
        self.text_size
    }
}
