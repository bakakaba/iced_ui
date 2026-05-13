//! Styling primitives for the [`Dialog`](super::Dialog) widget.

use iced::{Background, Border, Color, Shadow, Vector};

use crate::{Roundness, Theme};

/// The visual style of a [`Dialog`](super::Dialog).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Color of the scrim (semi-transparent overlay behind the dialog).
    pub scrim_color: Color,
    /// Background of the dialog container.
    pub background: Background,
    /// Border of the dialog container.
    pub border: Border,
    /// Shadow (elevation) of the dialog container.
    pub shadow: Shadow,
    /// Title text color.
    pub title_color: Color,
    /// Body text color.
    pub text_color: Color,
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Dialog`](super::Dialog).
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

/// The default dialog style.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(7.0));

    Style {
        scrim_color: Color::from_rgba(0.0, 0.0, 0.0, 0.32),
        background: Background::Color(palette.background.base.color),
        border: Border {
            radius: radius.into(),
            ..Border::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        title_color: palette.background.base.text,
        text_color: palette.background.weak.text,
    }
}
