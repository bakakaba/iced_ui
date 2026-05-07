//! The `iced_ui` theme.
//!
//! [`Theme`] is the default `Theme` parameter for every widget in this
//! crate. It composes a built-in [`iced::Theme`] (the colors source)
//! with a global `roundness` and `spacing`, all of which can be tweaked
//! at runtime to restyle every `iced_ui` widget at once.
//!
//! ```
//! use iced_ui::Theme;
//!
//! let theme = Theme {
//!     colors: iced::Theme::Dark,
//!     roundness: 8.0,
//!     spacing: 8.0,
//! };
//! assert_eq!(theme.roundness, 8.0);
//! ```
//!
//! Per-widget overrides (e.g. `Card::style`, `MenuBar::style`,
//! `Card::padding`) keep working and take precedence over the values
//! the theme provides.

mod iced_compat;

use iced::theme::palette;

/// The `iced_ui` theme.
///
/// `colors` carries the underlying [`iced::Theme`] used to derive every
/// theme-aware color (palette, extended palette). `roundness` is the
/// global border radius (in logical pixels) used for cards and menu
/// surfaces. `spacing` is the global spacing (in logical pixels) used
/// between menu items, inside menu rows, etc.
///
/// All three fields are public so application code can tweak them
/// directly. Construct one with [`Theme::default`] (Dark colors,
/// roundness `8`, spacing `8`) or by setting fields explicitly.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// The underlying built-in [`iced::Theme`] used as the source of
    /// truth for every color.
    pub colors: iced::Theme,
    /// Global border radius, in logical pixels. Drives card corners,
    /// menu popup corners and bar-item hover highlights.
    pub roundness: f32,
    /// Global spacing, in logical pixels. Drives the gap between
    /// top-level menu labels and the gutters inside dropdown rows.
    pub spacing: f32,
}

impl Theme {
    /// The default global border radius, in logical pixels.
    pub const DEFAULT_ROUNDNESS: f32 = 8.0;

    /// The default global spacing, in logical pixels.
    pub const DEFAULT_SPACING: f32 = 8.0;

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
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            colors: iced::Theme::Dark,
            roundness: Self::DEFAULT_ROUNDNESS,
            spacing: Self::DEFAULT_SPACING,
        }
    }
}

impl From<iced::Theme> for Theme {
    fn from(colors: iced::Theme) -> Self {
        Self {
            colors,
            roundness: Self::DEFAULT_ROUNDNESS,
            spacing: Self::DEFAULT_SPACING,
        }
    }
}
