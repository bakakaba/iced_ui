//! Styling primitives for the [`NavigationRail`](super::NavigationRail) widget.

use iced::{Background, Border, Color};

use crate::Theme;

/// The interaction status of a single destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DestinationStatus {
    /// The destination is currently selected/active.
    Active,
    /// The destination is not selected.
    Inactive,
    /// The pointer is hovering over the destination.
    Hovered,
}

/// The visual style of the [`NavigationRail`](super::NavigationRail) widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background fill for the rail strip.
    pub background: Option<Background>,
    /// Color of the active indicator pill.
    pub active_indicator_color: Color,
    /// Color for the active destination icon/text.
    pub active_icon_color: Color,
    /// Color for inactive destination icons/text.
    pub inactive_icon_color: Color,
    /// Color for destination labels.
    pub label_color: Color,
    /// Border around the rail.
    pub border: Border,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            active_indicator_color: Color::WHITE,
            active_icon_color: Color::WHITE,
            inactive_icon_color: Color {
                a: 0.6,
                ..Color::WHITE
            },
            label_color: Color::WHITE,
            border: Border::default(),
        }
    }
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`NavigationRail`](super::NavigationRail) widget.
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

/// The default navigation rail style.
///
/// - Surface background.
/// - Primary color for the active indicator and active icon.
/// - On-surface-variant for inactive icons.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: Some(Background::Color(palette.background.weak.color)),
        active_indicator_color: palette.primary.weak.color,
        active_icon_color: palette.primary.base.color,
        inactive_icon_color: palette.background.weak.text,
        label_color: palette.background.base.text,
        border: Border::default(),
    }
}
