//! Styling primitives for the [`Spinner`](super::Spinner) widget.

use crate::Theme;

/// A semantic color token used to tint a loading indicator.
///
/// Each variant maps to a color group on the active [`Theme`]:
/// [`Info`](Color::Info) resolves to the theme's
/// [`information`](crate::Theme::information) group, while the others
/// resolve to the matching groups on the
/// [`extended_palette`](crate::Theme::extended_palette).
///
/// Defaults to [`Info`](Color::Info).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// The theme's informational color (typically cyan).
    #[default]
    Info,
    /// The theme's primary brand color.
    Primary,
    /// The theme's success color (typically green).
    Success,
    /// The theme's warning color (typically amber).
    Warning,
    /// The theme's danger/error color (typically red).
    Danger,
}

impl Color {
    /// Resolves this token to a concrete color on the given theme.
    pub fn resolve(self, theme: &Theme) -> iced::Color {
        let palette = theme.extended_palette();
        match self {
            Self::Info => theme.information.base.color,
            Self::Primary => palette.primary.base.color,
            Self::Success => palette.success.base.color,
            Self::Warning => palette.warning.base.color,
            Self::Danger => palette.danger.base.color,
        }
    }
}

/// The visual style of a [`Spinner`](super::Spinner).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Color of the spinner's elements.
    pub color: iced::Color,
}

/// A function that returns a [`Style`] for a given theme and color
/// token.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Color) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Spinner`](super::Spinner).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class and color token.
    fn style(&self, class: &Self::Class<'_>, color: Color) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, color: Color) -> Style {
        class(self, color)
    }
}

/// The default spinner style: tints the elements with the resolved
/// [`Color`] token (defaulting to the theme's information color).
pub fn default(theme: &Theme, color: Color) -> Style {
    Style {
        color: color.resolve(theme),
    }
}
