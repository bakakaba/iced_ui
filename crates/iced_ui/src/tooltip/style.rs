//! Styling primitives for the [`Tooltip`](super::Tooltip) widget.

use iced::{Border, Color, Shadow};

use crate::{Elevation, Roundness, ShadowDir, Theme};

/// The visual style of a [`Tooltip`](super::Tooltip) bubble.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background color of the tooltip bubble.
    pub background: Color,
    /// Text color of the tooltip content.
    pub text_color: Color,
    /// Border drawn around the bubble.
    pub border: Border,
    /// Drop shadow cast by the bubble.
    pub shadow: Shadow,
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Tooltip`](super::Tooltip).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The default tooltip style: an elevated neutral surface with rounded
/// corners and a soft drop shadow.
pub fn default(theme: &Theme) -> Style {
    let neutral = theme.extended_palette().background.neutral;
    Style {
        background: neutral.color,
        text_color: neutral.text,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: theme.radius(Roundness::sx(1.0)).into(),
        },
        shadow: theme.shadow(Elevation::sx(1.0), ShadowDir::Down),
    }
}
