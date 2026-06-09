//! Shared positional anchor used by multiple widgets.
//!
//! [`Position`] describes where a child element (badge, snackbar, etc.)
//! is placed relative to its parent bounds.

use iced::{Point, Rectangle};

/// Positional anchor relative to a bounding rectangle.
///
/// Used by [`Badge`](crate::Badge) to place its indicator and by
/// [`Snackbar`](crate::Snackbar) to anchor the notification bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Position {
    /// Centered on the top edge.
    Top,
    /// Centered on the top-right corner.
    TopRight,
    /// Centered on the right edge.
    Right,
    /// Centered on the bottom-right corner.
    #[default]
    BottomRight,
    /// Centered on the bottom edge.
    Bottom,
    /// Centered on the bottom-left corner.
    BottomLeft,
    /// Centered on the left edge.
    Left,
    /// Centered on the top-left corner.
    TopLeft,
}

impl Position {
    /// Returns the anchor point on the given bounds for this position.
    pub fn anchor(self, bounds: &Rectangle) -> Point {
        match self {
            Self::Top => Point::new(bounds.x + bounds.width / 2.0, bounds.y),
            Self::TopRight => Point::new(bounds.x + bounds.width, bounds.y),
            Self::Right => Point::new(bounds.x + bounds.width, bounds.y + bounds.height / 2.0),
            Self::BottomRight => Point::new(bounds.x + bounds.width, bounds.y + bounds.height),
            Self::Bottom => Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height),
            Self::BottomLeft => Point::new(bounds.x, bounds.y + bounds.height),
            Self::Left => Point::new(bounds.x, bounds.y + bounds.height / 2.0),
            Self::TopLeft => Point::new(bounds.x, bounds.y),
        }
    }

    /// Whether this position is along the top edge.
    pub fn is_top(self) -> bool {
        matches!(self, Self::Top | Self::TopRight | Self::TopLeft)
    }

    /// Whether this position is along the bottom edge.
    pub fn is_bottom(self) -> bool {
        matches!(self, Self::Bottom | Self::BottomRight | Self::BottomLeft)
    }

    /// Whether this position is along the left edge.
    pub fn is_left(self) -> bool {
        matches!(self, Self::Left | Self::TopLeft | Self::BottomLeft)
    }

    /// Whether this position is along the right edge.
    pub fn is_right(self) -> bool {
        matches!(self, Self::Right | Self::TopRight | Self::BottomRight)
    }
}
