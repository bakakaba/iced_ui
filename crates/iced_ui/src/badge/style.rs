//! Styling primitives for the [`Badge`](super::Badge) widget.

use iced::{Color, Shadow};

use crate::{Elevation, ShadowDir, Theme};

/// The visual style of a [`Badge`](super::Badge).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background color of the badge indicator.
    pub background: Color,
    /// Text color of the badge count label. Only relevant for count
    /// badges.
    pub text_color: Color,
    /// Drop shadow cast by the badge indicator.
    pub shadow: Shadow,
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Badge`](super::Badge).
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

/// The default badge style: uses the theme's danger/error color for the
/// background and white text.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: palette.danger.base.color,
        text_color: palette.danger.base.text,
        shadow: theme.shadow(Elevation::sx(0.5), ShadowDir::Down),
    }
}
