//! Styling primitives for the [`MenuBar`](crate::menu::MenuBar).

use iced::{Background, Border, Color, Shadow};

use crate::{Elevation, Roundness, ShadowDir, Space, Theme};

/// The visual style of a [`MenuBar`](crate::menu::MenuBar) in a single
/// state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background of the horizontal bar.
    pub bar_background: Background,
    /// Border of the horizontal bar.
    pub bar_border: Border,
    /// Border radius applied to hover/active highlight backgrounds
    /// (both the top-level bar items and the rows inside dropdown
    /// popups), in logical pixels.
    pub item_radius: f32,

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
    /// Glyph drawn in the icon column of a [checked
    /// item](crate::menu::Item::checked).
    pub check_glyph: char,
    /// Color of the [`check_glyph`](Self::check_glyph) when the
    /// containing row is in its idle state. Hovered and disabled rows
    /// fall back to [`item_text_hovered`](Self::item_text_hovered) and
    /// [`item_text_disabled`](Self::item_text_disabled) respectively
    /// so the glyph stays consistent with the rest of the row.
    pub check_color: Color,

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
/// a [`Style`] from the theme's extended palette, with default
/// roundness and spacing multipliers.
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
        item_radius: theme.radius(Roundness::sx(1.0)),

        menu_background: palette.background.base.color.into(),
        menu_border: Border {
            radius: theme.radius(Roundness::sx(1.0)).into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        menu_shadow: theme.shadow(Elevation::sx(1.0), ShadowDir::Down),

        item_text: palette.background.base.text,
        item_text_disabled: palette.background.strong.color,
        item_background_hovered: palette.primary.base.color.into(),
        item_text_hovered: palette.primary.base.text,
        shortcut_text: palette.background.strong.color,
        separator_color: palette.background.strong.color,
        check_glyph: '\u{2713}',
        check_color: palette.primary.base.color,

        spacing: theme.space(Space::sx(1.0)),
    }
}
