//! Styling primitives for the [`Snackbar`](super::Snackbar) widget.

use iced::{Background, Border, Color, Shadow, Vector};

use crate::{Roundness, Theme};

/// The visual style of a [`Snackbar`](super::Snackbar).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background of the snackbar bar.
    pub background: Background,
    /// Message text color.
    pub text_color: Color,
    /// Action button text color.
    pub action_color: Color,
    /// Border of the snackbar bar.
    pub border: Border,
    /// Shadow (elevation) of the snackbar bar.
    pub shadow: Shadow,
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Snackbar`](super::Snackbar).
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

/// The default snackbar style.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(1.0));

    Style {
        background: Background::Color(Color::from_rgb(
            0.196, 0.196, 0.196, // ~#323232
        )),
        text_color: Color::WHITE,
        action_color: palette.primary.base.color,
        border: Border {
            radius: radius.into(),
            ..Border::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
    }
}
