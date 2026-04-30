//! Styling primitives for the [`MenuBar`](crate::menu::MenuBar).

use iced::{Background, Border, Color, Shadow, Theme};

/// The visual style of a [`MenuBar`](crate::menu::MenuBar) in a single
/// state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background of the horizontal bar.
    pub bar_background: Background,
    /// Border of the horizontal bar.
    pub bar_border: Border,
    /// Text color of top-level (bar) labels in their idle state.
    pub bar_text: Color,
    /// Background applied to a top-level label when it is hovered or
    /// when its dropdown is open.
    pub bar_item_background_active: Background,
    /// Text color applied to a top-level label when it is hovered or
    /// when its dropdown is open.
    pub bar_text_active: Color,

    /// Background of an open dropdown / submenu popup.
    pub menu_background: Background,
    /// Border of an open dropdown / submenu popup.
    pub menu_border: Border,
    /// Shadow of an open dropdown / submenu popup.
    pub menu_shadow: Shadow,

    /// Text color of an item in its idle state.
    pub item_text: Color,
    /// Text color of a disabled item.
    pub item_text_disabled: Color,
    /// Background of a hovered / highlighted item.
    pub item_background_hovered: Background,
    /// Text color of a hovered / highlighted item.
    pub item_text_hovered: Color,
    /// Color of the shortcut label on the right of an item.
    pub shortcut_text: Color,
    /// Color of a separator line between items.
    pub separator_color: Color,

    /// General spacing used by the menu, in logical pixels.
    ///
    /// Currently drives:
    ///
    /// - the horizontal gap between top-level menu labels in the bar, and
    /// - the interior gutters inside dropdown rows (icon ↔ label and
    ///   label ↔ shortcut).
    ///
    /// Increase this value for a roomier look; lower it for a denser one.
    pub spacing: f32,
}

/// A function that returns a [`Style`] for a given `Theme`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// A catalog of theme-driven [`Style`]s for the menu bar.
///
/// The default class — [`Catalog::default`] — uses [`default`] to derive
/// a [`Style`] from the theme's extended palette, matching the look of
/// the built-in iced widgets.
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] given the active theme and a class.
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

/// The default [`Style`] of a menu bar for the given [`Theme`].
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        bar_background: palette.background.weak.color.into(),
        bar_border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        bar_text: palette.background.weak.text,
        bar_item_background_active: palette.primary.base.color.into(),
        bar_text_active: palette.primary.base.text,

        menu_background: palette.background.base.color.into(),
        menu_border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        menu_shadow: Shadow {
            color: Color {
                a: 0.2,
                ..Color::BLACK
            },
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },

        item_text: palette.background.base.text,
        item_text_disabled: palette.background.strong.color,
        item_background_hovered: palette.primary.base.color.into(),
        item_text_hovered: palette.primary.base.text,
        shortcut_text: palette.background.strong.color,
        separator_color: palette.background.strong.color,

        spacing: 8.0,
    }
}
