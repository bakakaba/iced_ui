//! Small indicator overlaid on a child widget, showing either
//! a dot (no content) or a count value.
//!
//! The badge is positioned at the top-right corner of the child
//! content. A dot badge is a small filled circle; a count badge is a
//! pill-shaped container with a numeric label. Values exceeding the
//! configured maximum are displayed as "999+" (or similar).
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::badge::Badge;
//!
//! // Dot badge on an icon
//! let b = Badge::dot(icon_element);
//!
//! // Count badge
//! let b = Badge::count(icon_element, 5);
//! ```

mod style;

pub use style::{Catalog, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, Text, text};
use iced::alignment;
use iced::mouse;
use iced::{Element, Event, Length, Pixels, Point, Rectangle, Size};

use crate::SpacingBase;

/// The content shown inside the badge indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Content {
    /// A small dot with no text.
    Dot,
    /// A numeric count value.
    Count(u32),
}

/// A badge overlaid at the top-right corner of a child widget.
pub struct Badge<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + text::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    badge_content: Content,
    max: u32,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Badge<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + text::Renderer,
{
    /// Creates a dot badge overlaid on the given content.
    pub fn dot(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            badge_content: Content::Dot,
            max: 999,
            class: Theme::default(),
        }
    }

    /// Creates a count badge overlaid on the given content.
    pub fn count(content: impl Into<Element<'a, Message, Theme, Renderer>>, value: u32) -> Self {
        Self {
            content: content.into(),
            badge_content: Content::Count(value),
            max: 999,
            class: Theme::default(),
        }
    }

    /// Sets the maximum displayed count. Values above this show as
    /// "{max}+". Defaults to 999.
    pub fn max(mut self, max: u32) -> Self {
        self.max = max;
        self
    }

    /// Sets the style class for this badge.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Badge<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + SpacingBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        // Draw the child content first.
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        let badge_style = Catalog::style(theme, &self.class);
        let bounds = layout.bounds();

        match self.badge_content {
            Content::Dot => {
                let dot_size = 6.0_f32;
                let dot_rect = Rectangle {
                    x: bounds.x + bounds.width - dot_size,
                    y: bounds.y,
                    width: dot_size,
                    height: dot_size,
                };

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: dot_rect,
                        border: iced::Border {
                            radius: (dot_size / 2.0).into(),
                            ..iced::Border::default()
                        },
                        shadow: iced::Shadow::default(),
                        ..renderer::Quad::default()
                    },
                    iced::Background::Color(badge_style.background),
                );
            }
            Content::Count(value) => {
                if value == 0 {
                    return;
                }

                let label = if value > self.max {
                    format!("{}+", self.max)
                } else {
                    value.to_string()
                };

                let font_size = 11.0_f32;
                let h_padding = 4.0_f32;
                let min_width = 16.0_f32;
                let height = 16.0_f32;

                // Measure text width
                let text_width = label.len() as f32 * font_size * 0.6;
                let badge_width = (text_width + h_padding * 2.0).max(min_width);
                let radius = height / 2.0;

                let badge_rect = Rectangle {
                    x: bounds.x + bounds.width - badge_width * 0.6,
                    y: bounds.y - height * 0.3,
                    width: badge_width,
                    height,
                };

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: badge_rect,
                        border: iced::Border {
                            radius: radius.into(),
                            ..iced::Border::default()
                        },
                        shadow: iced::Shadow::default(),
                        ..renderer::Quad::default()
                    },
                    iced::Background::Color(badge_style.background),
                );

                // Draw the count text centered in the badge.
                renderer.fill_text(
                    Text {
                        content: label,
                        bounds: Size::new(badge_rect.width, badge_rect.height),
                        size: Pixels(font_size),
                        line_height: text::LineHeight::Relative(1.0),
                        font: renderer.default_font(),
                        align_x: alignment::Horizontal::Center.into(),
                        align_y: alignment::Vertical::Center,
                        shaping: text::Shaping::Basic,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(
                        badge_rect.x + badge_rect.width / 2.0,
                        badge_rect.y + badge_rect.height / 2.0,
                    ),
                    badge_style.text_color,
                    badge_rect,
                );
            }
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
}

impl<'a, Message, Theme, Renderer> From<Badge<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + SpacingBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn from(badge: Badge<'a, Message, Theme, Renderer>) -> Self {
        Element::new(badge)
    }
}
