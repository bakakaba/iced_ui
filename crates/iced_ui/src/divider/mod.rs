//! Thin horizontal or vertical line that separates content with
//! optional leading/trailing insets.
//!
//! Unlike iced's built-in [`Rule`](iced::widget::Rule), a [`Divider`]
//! supports themed insets (leading space before the line starts) and
//! integrates with the `iced_ui` [`Catalog`] trait pattern for
//! consistent theming.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::divider::Divider;
//!
//! let d = Divider::horizontal();
//! let d_inset = Divider::horizontal().inset_leading(Space::sx(4.0));
//! ```

mod style;

pub use style::{Catalog, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::mouse;
use iced::{Element, Length, Rectangle, Size};

use crate::{Space, SpacingBase};

/// Orientation of a divider line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    /// A horizontal line spanning the available width.
    #[default]
    Horizontal,
    /// A vertical line spanning the available height.
    Vertical,
}

/// A thin line separator between content sections.
///
/// Construct with [`Divider::horizontal`] or [`Divider::vertical`].
pub struct Divider<'a, Theme = crate::Theme>
where
    Theme: Catalog,
{
    orientation: Orientation,
    inset_leading: Option<Space>,
    inset_trailing: Option<Space>,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Divider<'a, Theme>
where
    Theme: Catalog,
{
    /// Creates a horizontal divider.
    pub fn horizontal() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            inset_leading: None,
            inset_trailing: None,
            class: Theme::default(),
        }
    }

    /// Creates a vertical divider.
    pub fn vertical() -> Self {
        Self {
            orientation: Orientation::Vertical,
            inset_leading: None,
            inset_trailing: None,
            class: Theme::default(),
        }
    }

    /// Sets the leading inset (left for horizontal, top for vertical).
    pub fn inset_leading(mut self, space: Space) -> Self {
        self.inset_leading = Some(space);
        self
    }

    /// Sets the trailing inset (right for horizontal, bottom for
    /// vertical).
    pub fn inset_trailing(mut self, space: Space) -> Self {
        self.inset_trailing = Some(space);
        self
    }

    /// Sets both leading and trailing insets to the same value.
    pub fn inset(self, space: Space) -> Self {
        self.inset_leading(space).inset_trailing(space)
    }

    /// Sets the style class for this divider.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Divider<'a, Theme>
where
    Theme: Catalog + SpacingBase,
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        match self.orientation {
            Orientation::Horizontal => Size::new(Length::Fill, Length::Shrink),
            Orientation::Vertical => Size::new(Length::Shrink, Length::Fill),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        match self.orientation {
            Orientation::Horizontal => {
                let max = limits.max();
                layout::Node::new(Size::new(max.width, 1.0))
            }
            Orientation::Vertical => {
                let max = limits.max();
                layout::Node::new(Size::new(1.0, max.height))
            }
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let style = Catalog::style(theme, &self.class);
        let bounds = layout.bounds();
        let spacing = theme.spacing();

        let leading = self
            .inset_leading
            .map(|s| s.resolve(spacing))
            .unwrap_or(0.0);
        let trailing = self
            .inset_trailing
            .map(|s| s.resolve(spacing))
            .unwrap_or(0.0);

        let rect = match self.orientation {
            Orientation::Horizontal => Rectangle {
                x: bounds.x + leading,
                y: bounds.y,
                width: (bounds.width - leading - trailing).max(0.0),
                height: style.thickness,
            },
            Orientation::Vertical => Rectangle {
                x: bounds.x,
                y: bounds.y + leading,
                width: style.thickness,
                height: (bounds.height - leading - trailing).max(0.0),
            },
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: rect,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                ..renderer::Quad::default()
            },
            iced::Background::Color(style.color),
        );
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<'a, Message, Theme, Renderer> From<Divider<'a, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + SpacingBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(divider: Divider<'a, Theme>) -> Self {
        Element::new(divider)
    }
}
