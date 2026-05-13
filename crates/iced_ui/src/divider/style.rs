//! Styling primitives for the [`Divider`](super::Divider) widget.

use iced::Color;

use crate::Theme;

/// The visual style of a [`Divider`](super::Divider).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Color of the divider line.
    pub color: Color,
    /// Thickness of the divider line in logical pixels.
    pub thickness: f32,
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Divider`](super::Divider).
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

/// The default divider style: a thin line using the theme's background
/// strong color at reduced opacity.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        color: palette.background.strong.color,
        thickness: 1.0,
    }
}
