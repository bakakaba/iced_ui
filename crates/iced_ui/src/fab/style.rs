//! Styling primitives for the [`Fab`](super::Fab) widget.

use iced::{Background, Border, Color, Shadow, Vector};

use crate::{Roundness, Theme};

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
/// on-container for the icon. The "lowered" variant reduces the
/// shadow.
pub fn default(theme: &Theme, lowered: bool, status: Status) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(4.0));

    let shadow_offset = if lowered { 1.0 } else { 3.0 };
    let shadow_blur = if lowered { 2.0 } else { 6.0 };

    let base = Style {
        background: Background::Color(palette.primary.weak.color),
        icon_color: palette.primary.base.color,
        border: Border {
            radius: radius.into(),
            ..Border::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, shadow_offset),
            blur_radius: shadow_blur,
        },
    };

    match status {
        Status::Active => base,
        Status::Hovered => Style {
            shadow: Shadow {
                offset: Vector::new(0.0, shadow_offset + 2.0),
                blur_radius: shadow_blur + 4.0,
                ..base.shadow
            },
            ..base
        },
        Status::Pressed => Style {
            shadow: Shadow {
                offset: Vector::new(0.0, shadow_offset),
                blur_radius: shadow_blur,
                ..base.shadow
            },
            ..base
        },
    }
}
