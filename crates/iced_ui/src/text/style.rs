//! Styling primitives for the [`Text`](super::Text) widget.

use iced::Color;

use crate::Theme;

/// The visual style of a [`Text`](super::Text) heading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Text color. `None` inherits the renderer's default text color.
    pub color: Option<Color>,
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Text`](super::Text).
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

/// The default text style: inherits the renderer's default text color.
pub fn default(_theme: &Theme) -> Style {
    Style { color: None }
}
