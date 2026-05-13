//! Styling primitives for the [`TopAppBar`](super::TopAppBar) widget.

use iced::{Background, Border, Color};

use crate::Theme;

/// The visual style of a [`TopAppBar`](super::TopAppBar).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background fill for the app bar.
    pub background: Background,
    /// Color for the title text.
    pub title_color: Color,
    /// Color for icon elements (navigation and actions).
    pub icon_color: Color,
    /// Border around the app bar.
    pub border: Border,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: Background::Color(Color::TRANSPARENT),
            title_color: Color::WHITE,
            icon_color: Color::WHITE,
            border: Border::default(),
        }
    }
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`TopAppBar`](super::TopAppBar).
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

/// The default top app bar style.
///
/// - Surface background.
/// - On-surface color for the title text.
/// - Primary color for icons.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: Background::Color(palette.background.weak.color),
        title_color: palette.background.base.text,
        icon_color: palette.primary.base.color,
        border: Border::default(),
    }
}
