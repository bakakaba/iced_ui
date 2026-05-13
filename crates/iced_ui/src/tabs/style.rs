//! Styling primitives for the [`Tabs`](super::Tabs) widget.

use iced::{Background, Border, Color};

use crate::Theme;

/// The interaction status of a single tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabStatus {
    /// The tab is currently selected/active.
    Active,
    /// The tab is not selected.
    Inactive,
    /// The pointer is hovering over the tab.
    Hovered,
}

/// The visual style of the [`Tabs`](super::Tabs) widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background fill for the entire tab row.
    pub background: Option<Background>,
    /// Color of the active indicator line.
    pub active_indicator_color: Color,
    /// Text color for the active tab.
    pub active_text_color: Color,
    /// Text color for inactive tabs.
    pub inactive_text_color: Color,
    /// Border around the tab row.
    pub border: Border,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            active_indicator_color: Color::WHITE,
            active_text_color: Color::WHITE,
            inactive_text_color: Color {
                a: 0.6,
                ..Color::WHITE
            },
            border: Border::default(),
        }
    }
}

/// A function that returns a [`Style`] for a given theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Tabs`](super::Tabs) widget.
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

/// The default tab style.
///
/// - No background fill.
/// - Primary color for the active indicator and active text.
/// - Muted (on-surface with reduced alpha) for inactive text.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: None,
        active_indicator_color: palette.primary.base.color,
        active_text_color: palette.primary.base.color,
        inactive_text_color: palette.background.weak.text,
        border: Border::default(),
    }
}
