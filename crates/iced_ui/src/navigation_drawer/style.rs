//! Styling primitives for the [`NavigationDrawer`](super::NavigationDrawer) widget.

use iced::{Background, Border, Color, Shadow};

use crate::{Elevation, Roundness, ShadowDir, Theme};

/// The visual style of a [`NavigationDrawer`](super::NavigationDrawer).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background of the drawer panel.
    pub background: Background,
    /// Color of the scrim (semi-transparent overlay behind the drawer in modal mode).
    pub scrim_color: Color,
    /// Background color of the active destination indicator pill.
    pub active_indicator_color: Color,
    /// Text color of the active destination.
    pub active_text_color: Color,
    /// Text color of inactive destinations.
    pub inactive_text_color: Color,
    /// Border of the drawer panel.
    pub border: Border,
    /// Shadow (elevation) of the drawer panel.
    pub shadow: Shadow,
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`NavigationDrawer`](super::NavigationDrawer).
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

/// The default navigation drawer style.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(2.0));

    Style {
        background: Background::Color(palette.background.base.color),
        scrim_color: Color::from_rgba(0.0, 0.0, 0.0, 0.32),
        active_indicator_color: palette.primary.weak.color,
        active_text_color: palette.primary.base.color,
        inactive_text_color: palette.background.weak.text,
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: radius,
                bottom_right: radius,
                bottom_left: 0.0,
            },
            ..Border::default()
        },
        shadow: theme.shadow(Elevation::sx(1.0), ShadowDir::Right),
    }
}
