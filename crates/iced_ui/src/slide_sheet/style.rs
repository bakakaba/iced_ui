//! Styling primitives for the [`SlideSheet`](super::SlideSheet) widget.

use iced::{Background, Border, Color, Shadow};

use crate::{Elevation, Roundness, ShadowDir, Theme};

/// The visual style of a [`SlideSheet`](super::SlideSheet).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background of the sheet panel.
    pub background: Background,
    /// Border of the sheet panel.
    pub border: Border,
    /// Shadow (elevation) of the sheet panel.
    pub shadow: Shadow,
    /// Color of the drag handle pill.
    pub handle_color: Color,
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`SlideSheet`](super::SlideSheet).
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

/// Default theme style.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(3.0));

    Style {
        background: Background::Color(palette.background.base.color),
        border: Border {
            radius: iced::border::Radius {
                top_left: radius,
                top_right: radius,
                bottom_right: radius,
                bottom_left: radius,
            },
            ..Border::default()
        },
        // Base shadow; the widget overrides the offset direction at
        // draw time based on the sheet's anchor.
        shadow: theme.shadow(Elevation::sx(1.0), ShadowDir::Down),
        handle_color: palette.background.strong.color,
    }
}
