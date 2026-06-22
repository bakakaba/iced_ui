//! Styling primitives for the [`Tree`](crate::tree::Tree) widget.
//!
//! Each visible node row in a tree has a [`Status`] that indicates its
//! current interaction state (idle, hovered, or pressed). The
//! [`Catalog`] trait maps a theme and status to an [`ItemStyle`],
//! allowing full control over per-row appearance.

use iced::{Background, Border, Color};

use crate::Theme;

/// The interaction status of a single tree node row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The row is idle — no pointer interaction.
    Active,
    /// The pointer is hovering over the row.
    Hovered,
    /// The pointer is pressing the row (mouse button held down).
    Pressed,
}

/// The visual style applied to a single tree node row based on its
/// [`Status`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ItemStyle {
    /// Background of the node row.
    pub background: Option<Background>,
    /// Border around the node row.
    pub border: Border,
    /// Text color for the row. `None` inherits from the parent.
    pub text_color: Option<Color>,
}

/// A function that returns an [`ItemStyle`] for a given theme and
/// [`Status`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> ItemStyle + 'a>;

/// A catalog of theme-driven [`ItemStyle`]s for the tree.
///
/// The default class — [`Catalog::default`] — uses [`default`] to
/// derive an [`ItemStyle`] from the theme's extended palette for each
/// [`Status`].
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves an [`ItemStyle`] given the active theme, class and
    /// status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> ItemStyle;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> ItemStyle {
        class(self, status)
    }
}

/// The default [`ItemStyle`] of a tree node row for the given [`Theme`]
/// and [`Status`].
///
/// - **Active**: transparent background, no border.
/// - **Hovered**: subtle background highlight from the extended
///   palette.
/// - **Pressed**: slightly stronger background highlight.
pub fn default(theme: &Theme, status: Status) -> ItemStyle {
    let palette = theme.extended_palette();

    match status {
        Status::Active => ItemStyle {
            background: None,
            border: Border::default(),
            text_color: None,
        },
        Status::Hovered => ItemStyle {
            background: Some(Background::Color(palette.background.weak.color)),
            border: Border::default(),
            text_color: None,
        },
        Status::Pressed => ItemStyle {
            background: Some(Background::Color(palette.background.strong.color)),
            border: Border::default(),
            text_color: None,
        },
    }
}
