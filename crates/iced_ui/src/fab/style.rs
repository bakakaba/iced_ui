//! Styling primitives for the [`Fab`](super::Fab) widget.

use iced::{Background, Border, Color, Shadow};

use crate::{Elevation, ShadowDir, Theme};

/// The size variant of a FAB.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FabSize {
    /// 40x40 logical pixels.
    Small,
    /// 56x56 logical pixels. This is the default.
    #[default]
    Regular,
    /// 96x96 logical pixels.
    Large,
}

impl FabSize {
    /// Returns the side length in logical pixels.
    pub fn pixels(self) -> f32 {
        match self {
            Self::Small => 40.0,
            Self::Regular => 56.0,
            Self::Large => 96.0,
        }
    }

    /// Returns the icon size in logical pixels.
    pub fn icon_size(self) -> f32 {
        match self {
            Self::Small => 24.0,
            Self::Regular => 24.0,
            Self::Large => 36.0,
        }
    }
}

/// The interaction status of a FAB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Normal state.
    Active,
    /// Pointer hovering.
    Hovered,
    /// Being pressed.
    Pressed,
}

/// The visual style of a [`Fab`](super::Fab).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background fill.
    pub background: Background,
    /// Icon/text color.
    pub icon_color: Color,
    /// Border.
    pub border: Border,
    /// Shadow (elevation).
    pub shadow: Shadow,
}

/// A function that returns a [`Style`] for a given theme, lowered
/// state, and interaction status.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, bool, Status) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Fab`](super::Fab).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class, lowered flag, and
    /// status.
    fn style(&self, class: &Self::Class<'_>, lowered: bool, status: Status) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, lowered: bool, status: Status) -> Style {
        class(self, lowered, status)
    }
}

/// The default FAB style.
///
/// Uses the primary container color for the background and primary
/// on-container for the icon. The drop shadow is driven entirely by the
/// theme's [`elevation`](crate::Theme::elevation) and does not change
/// with the lowered flag or interaction status.
pub fn default(theme: &Theme, _lowered: bool, _status: Status) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: Background::Color(palette.primary.weak.color),
        icon_color: palette.primary.base.color,
        // The corner radius is resolved by the widget at draw time
        // (fully rounded by default, or a `.roundness(..)` override),
        // so it is left at the default here.
        border: Border::default(),
        shadow: theme.shadow(Elevation::sx(1.0), ShadowDir::Down),
    }
}
