//! Custom color tokens that extend iced's built-in palette.
//!
//! These tokens live on [`Theme`](super::Theme) alongside the standard
//! `iced::Theme` colors and are regenerated whenever the palette
//! changes.

use iced::Color;
use iced::theme::palette::{self, Pair};

/// Default information base color for dark themes.
pub const INFORMATION_DARK: Color = Color::from_rgb(0.2, 0.75, 0.95);

/// Default information base color for light themes.
pub const INFORMATION_LIGHT: Color = Color::from_rgb(0.0, 0.48, 0.65);

/// A set of paper (surface) colors — the inverse of background.
///
/// Paper represents surfaces that float above the background (cards,
/// sheets, dialogs). It uses the palette's text color as its base
/// (the visual inverse of the background) and generates 8 intensity
/// levels using the same deviation factors as iced's `Background`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Paper {
    /// The base paper color.
    pub base: Pair,
    /// The weakest deviation from base.
    pub weakest: Pair,
    /// A weaker deviation from base.
    pub weaker: Pair,
    /// A weak deviation from base.
    pub weak: Pair,
    /// A neutral deviation (between weak and strong).
    pub neutral: Pair,
    /// A strong deviation from base.
    pub strong: Pair,
    /// A stronger deviation from base.
    pub stronger: Pair,
    /// The strongest deviation from base.
    pub strongest: Pair,
}

impl Paper {
    /// Generates a [`Paper`] set from the palette's background and
    /// text colors.
    ///
    /// The base paper color is `text` (the inverse of background).
    /// The readable text color on paper surfaces is derived via
    /// [`Pair::new`] which internally calls `readable()` to guarantee
    /// contrast.
    pub fn generate(background: Color, text: Color) -> Self {
        let base = text;
        let weakest = palette::deviate(base, 0.03);
        let weaker = palette::deviate(base, 0.07);
        let weak = palette::deviate(base, 0.10);
        let neutral = palette::deviate(base, 0.125);
        let strong = palette::deviate(base, 0.15);
        let stronger = palette::deviate(base, 0.175);
        let strongest = palette::deviate(base, 0.20);

        Self {
            base: Pair::new(base, background),
            weakest: Pair::new(weakest, background),
            weaker: Pair::new(weaker, background),
            weak: Pair::new(weak, background),
            neutral: Pair::new(neutral, background),
            strong: Pair::new(strong, background),
            stronger: Pair::new(stronger, background),
            strongest: Pair::new(strongest, background),
        }
    }
}

/// A set of informational colors (typically cyan).
///
/// Follows the same pattern as iced's `Primary`/`Success`/`Danger`
/// groups: a base color, a weaker variant mixed toward the
/// background, and a stronger variant deviated from the base.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Information {
    /// The base information color.
    pub base: Pair,
    /// A weaker version of the base information color.
    pub weak: Pair,
    /// A stronger version of the base information color.
    pub strong: Pair,
}

impl Information {
    /// Generates an [`Information`] color group from a base
    /// information color, the palette background, and text colors.
    pub fn generate(base: Color, background: Color, text: Color) -> Self {
        let weak = palette::mix(base, background, 0.4);
        let strong = palette::deviate(base, 0.1);

        Self {
            base: Pair::new(base, text),
            weak: Pair::new(weak, text),
            strong: Pair::new(strong, text),
        }
    }

    /// Returns the default base information color for the given
    /// palette, choosing between [`INFORMATION_DARK`] and
    /// [`INFORMATION_LIGHT`] based on whether the background is dark.
    pub fn default_base(background: Color) -> Color {
        if palette::is_dark(background) {
            INFORMATION_DARK
        } else {
            INFORMATION_LIGHT
        }
    }
}
