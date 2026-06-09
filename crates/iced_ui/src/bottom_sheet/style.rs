//! Styling primitives for the [`BottomSheet`](super::BottomSheet) widget.

use iced::{Background, Border, Color, Shadow, Vector};

use crate::{Roundness, Theme};

/// The visual style of a [`BottomSheet`](super::BottomSheet).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background of the sheet panel.
    pub background: Background,
    /// Color of the scrim (semi-transparent overlay behind the sheet).
    pub scrim_color: Color,
    /// Border of the sheet panel.
    pub border: Border,
    /// Shadow (elevation) of the sheet panel.
    pub shadow: Shadow,
    /// Color of the drag handle pill.
    pub handle_color: Color,
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`BottomSheet`](super::BottomSheet).
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

/// Default theme
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(4.0));

    Style {
        background: Background::Color(palette.background.base.color),
        scrim_color: Color::from_rgba(0.0, 0.0, 0.0, 0.32),
        border: Border {
            radius: iced::border::Radius {
                top_left: radius,
                top_right: radius,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            ..Border::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, -2.0),
            blur_radius: 12.0,
        },
        handle_color: palette.background.strong.color,
    }
}
