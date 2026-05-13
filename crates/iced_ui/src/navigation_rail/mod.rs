//! Vertical strip with 3–7 icon+label destinations for desktop/tablet layouts.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::navigation_rail::{Destination, NavigationRail};
//!
//! let rail = NavigationRail::new(Message::NavSelected)
//!     .push(Destination::new("Home"))
//!     .push(Destination::new("Search"))
//!     .push(Destination::new("Settings"))
//!     .active(selected_index);
//! ```

mod style;

pub use style::{Catalog, DestinationStatus, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use std::marker::PhantomData;

use iced::{Background, Color, Element, Event, Length, Rectangle, Size};

/// Width of the rail in logical pixels (MD3 spec).
const RAIL_WIDTH: f32 = 80.0;

/// Height of a single destination cell in logical pixels.
const DESTINATION_HEIGHT: f32 = 56.0;

/// Width of the active indicator pill.
const INDICATOR_WIDTH: f32 = 64.0;

/// Height of the active indicator pill.
const INDICATOR_HEIGHT: f32 = 32.0;

/// A single destination within a [`NavigationRail`].
///
/// Contains a text label.
pub struct Destination {
    label: String,
}

impl Destination {
    /// Creates a new destination with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

/// Internal per-destination interaction state.
#[derive(Debug, Clone, Default)]
struct DestinationState {
    is_hovered: bool,
}

/// Internal state for the whole navigation rail.
#[derive(Debug, Clone, Default)]
struct State {
    destinations: Vec<DestinationState>,
}

/// Renders as a vertical column of equal-height destination cells. The
/// active destination is highlighted with a pill-shaped indicator
/// behind the icon area.
pub struct NavigationRail<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    destinations: Vec<Destination>,
    active: usize,
    on_select: Box<dyn Fn(usize) -> Message + 'a>,
    class: Theme::Class<'a>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer> NavigationRail<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    /// Creates a new [`NavigationRail`] with the given selection callback.
    ///
    /// The callback receives the zero-based index of the selected destination.
    pub fn new(on_select: impl Fn(usize) -> Message + 'a) -> Self {
        Self {
            destinations: Vec::new(),
            active: 0,
            on_select: Box::new(on_select),
            class: Theme::default(),
            _renderer: PhantomData,
        }
    }

    /// Appends a [`Destination`] to the rail.
    pub fn push(mut self, destination: Destination) -> Self {
        self.destinations.push(destination);
        self
    }

    /// Sets the index of the currently active destination.
    pub fn active(mut self, index: usize) -> Self {
        self.active = index;
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for NavigationRail<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            destinations: vec![DestinationState::default(); self.destinations.len()],
        })
    }

    fn children(&self) -> Vec<Tree> {
        Vec::new()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        state
            .destinations
            .resize_with(self.destinations.len(), Default::default);
    }

    fn size(&self) -> Size<Length> {
        let total_height = DESTINATION_HEIGHT * self.destinations.len() as f32;
        Size::new(Length::Fixed(RAIL_WIDTH), Length::Fixed(total_height))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        let count = self.destinations.len();
        if count == 0 {
            return layout::Node::new(Size::new(RAIL_WIDTH, 0.0));
        }

        let total_height = DESTINATION_HEIGHT * count as f32;

        let mut children = Vec::with_capacity(count);
        for i in 0..count {
            let mut node = layout::Node::new(Size::new(RAIL_WIDTH, DESTINATION_HEIGHT));
            node = node.move_to(iced::Point::new(0.0, DESTINATION_HEIGHT * i as f32));
            children.push(node);
        }

        layout::Node::with_children(Size::new(RAIL_WIDTH, total_height), children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let style = Catalog::style(theme, &self.class);
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        // Draw the rail background if set.
        if let Some(bg) = style.background {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    ..renderer::Quad::default()
                },
                bg,
            );
        }

        // Draw each destination cell.
        for (i, (dest, cell_layout)) in self.destinations.iter().zip(layout.children()).enumerate()
        {
            let cell_bounds = cell_layout.bounds();
            let is_active = i == self.active;
            let dest_state = state.destinations.get(i).cloned().unwrap_or_default();

            let _status = if is_active {
                DestinationStatus::Active
            } else if dest_state.is_hovered {
                DestinationStatus::Hovered
            } else {
                DestinationStatus::Inactive
            };

            // Hover overlay.
            if dest_state.is_hovered && !is_active {
                let hover_color = Color {
                    a: 0.08,
                    ..style.active_indicator_color
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: cell_bounds,
                        ..renderer::Quad::default()
                    },
                    Background::Color(hover_color),
                );
            }

            // Active indicator pill (centered horizontally, at top of cell).
            if is_active {
                let pill_x = cell_bounds.x + (cell_bounds.width - INDICATOR_WIDTH) / 2.0;
                let pill_y = cell_bounds.y + 4.0;
                let pill_bounds = Rectangle {
                    x: pill_x,
                    y: pill_y,
                    width: INDICATOR_WIDTH,
                    height: INDICATOR_HEIGHT,
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: pill_bounds,
                        border: iced::Border {
                            radius: (INDICATOR_HEIGHT / 2.0).into(),
                            ..Default::default()
                        },
                        ..renderer::Quad::default()
                    },
                    Background::Color(style.active_indicator_color),
                );
            }

            // Label text — centered in cell, below the indicator area.
            let text_color = if is_active {
                style.active_icon_color
            } else {
                style.inactive_icon_color
            };

            let label_y = cell_bounds.y + INDICATOR_HEIGHT + 8.0;
            let label_height = cell_bounds.height - INDICATOR_HEIGHT - 8.0;

            let text = iced::advanced::text::Text {
                content: dest.label.clone(),
                bounds: Size::new(cell_bounds.width, label_height),
                size: renderer.default_size() * 0.85,
                line_height: iced::advanced::text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Top,
                shaping: iced::advanced::text::Shaping::Advanced,
                wrapping: iced::advanced::text::Wrapping::None,
            };

            renderer.fill_text(
                text,
                iced::Point::new(cell_bounds.center_x(), label_y),
                text_color,
                cell_bounds,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        state
            .destinations
            .resize_with(self.destinations.len(), Default::default);

        for (i, cell_layout) in layout.children().enumerate() {
            let bounds = cell_layout.bounds();
            let is_over = cursor.is_over(bounds);

            if let Some(dest_state) = state.destinations.get_mut(i) {
                match event {
                    Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                        dest_state.is_hovered = is_over;
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if is_over => {
                        shell.publish((self.on_select)(i));
                    }
                    _ => {}
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        for cell_layout in layout.children() {
            if cursor.is_over(cell_layout.bounds()) {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }

    fn operate(
        &mut self,
        _tree: &mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        // No child elements to propagate to.
    }
}

impl<'a, Message, Theme, Renderer> From<NavigationRail<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn from(rail: NavigationRail<'a, Message, Theme, Renderer>) -> Self {
        Element::new(rail)
    }
}
