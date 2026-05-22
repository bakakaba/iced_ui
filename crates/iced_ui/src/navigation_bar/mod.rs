//! Horizontal bottom bar with 3–5 destinations for mobile layouts.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::navigation_bar::{Destination, NavigationBar};
//!
//! let bar = NavigationBar::new(Message::NavSelected)
//!     .push(Destination::new("Home").icon(text("H").font(MY_ICON_FONT)))
//!     .push(Destination::new("Search").icon(text("S").font(MY_ICON_FONT)))
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

/// Height of the bar in logical pixels (MD3 spec).
const BAR_HEIGHT: f32 = 80.0;

/// Width of the active indicator pill.
const INDICATOR_WIDTH: f32 = 64.0;

/// Height of the active indicator pill.
const INDICATOR_HEIGHT: f32 = 32.0;

/// Size allocated for the icon within the pill.
const ICON_SIZE: f32 = 24.0;

/// A single destination within a [`NavigationBar`].
///
/// Contains a text label and an optional icon element.
pub struct Destination<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer> {
    label: String,
    icon: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Destination<'a, Message, Theme, Renderer> {
    /// Creates a new destination with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    /// Sets the icon element for this destination.
    ///
    /// The icon is rendered centered inside the pill indicator area,
    /// above the text label. Any iced widget can be used — typically a
    /// `text()` glyph from an icon font.
    pub fn icon(mut self, icon: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Internal per-destination interaction state.
#[derive(Debug, Clone, Default)]
struct DestinationState {
    is_hovered: bool,
}

/// Internal state for the whole navigation bar.
#[derive(Debug, Clone, Default)]
struct State {
    destinations: Vec<DestinationState>,
}

/// Renders as a horizontal row of equal-width destination cells. The
/// active destination is highlighted with a pill-shaped indicator
/// behind the icon area.
pub struct NavigationBar<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    destinations: Vec<Destination<'a, Message, Theme, Renderer>>,
    active: usize,
    on_select: Box<dyn Fn(usize) -> Message + 'a>,
    class: Theme::Class<'a>,
    width: Length,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer> NavigationBar<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    /// Creates a new [`NavigationBar`] with the given selection callback.
    ///
    /// The callback receives the zero-based index of the selected destination.
    pub fn new(on_select: impl Fn(usize) -> Message + 'a) -> Self {
        Self {
            destinations: Vec::new(),
            active: 0,
            on_select: Box::new(on_select),
            class: Theme::default(),
            width: Length::Fill,
            _renderer: PhantomData,
        }
    }

    /// Appends a [`Destination`] to the bar.
    pub fn push(mut self, destination: Destination<'a, Message, Theme, Renderer>) -> Self {
        self.destinations.push(destination);
        self
    }

    /// Sets the index of the currently active destination.
    pub fn active(mut self, index: usize) -> Self {
        self.active = index;
        self
    }

    /// Sets the width of the navigation bar.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for NavigationBar<'a, Message, Theme, Renderer>
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
        self.destinations
            .iter()
            .map(|dest| match &dest.icon {
                Some(icon) => Tree::new(icon),
                None => Tree::empty(),
            })
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        state
            .destinations
            .resize_with(self.destinations.len(), Default::default);

        // Diff icon children.
        tree.children
            .resize_with(self.destinations.len(), Tree::empty);
        for (i, dest) in self.destinations.iter().enumerate() {
            if let Some(icon) = &dest.icon {
                tree.children[i].diff(icon);
            }
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(BAR_HEIGHT))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let count = self.destinations.len();
        if count == 0 {
            return layout::Node::new(Size::new(0.0, BAR_HEIGHT));
        }

        let total_width = limits.resolve(self.width, Length::Shrink, Size::ZERO).width;
        let cell_width = total_width / count as f32;

        let mut children = Vec::with_capacity(count);
        for i in 0..count {
            // Layout the icon if present.
            let icon_child = if let Some(icon) = &mut self.destinations[i].icon {
                let icon_limits = layout::Limits::new(Size::ZERO, Size::new(ICON_SIZE, ICON_SIZE));
                let icon_node =
                    icon.as_widget_mut()
                        .layout(&mut tree.children[i], renderer, &icon_limits);
                Some(icon_node)
            } else {
                None
            };

            let mut cell_children = Vec::new();

            // Position the icon centered within the pill area.
            if let Some(mut icon_node) = icon_child {
                let icon_w = icon_node.size().width;
                let icon_h = icon_node.size().height;
                let icon_x = (cell_width - icon_w) / 2.0;
                let icon_y = 12.0 + (INDICATOR_HEIGHT - icon_h) / 2.0;
                icon_node = icon_node.move_to(iced::Point::new(icon_x, icon_y));
                cell_children.push(icon_node);
            }

            let mut cell_node =
                layout::Node::with_children(Size::new(cell_width, BAR_HEIGHT), cell_children);
            cell_node = cell_node.move_to(iced::Point::new(cell_width * i as f32, 0.0));
            children.push(cell_node);
        }

        layout::Node::with_children(Size::new(total_width, BAR_HEIGHT), children)
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
        let nav_style = Catalog::style(theme, &self.class);
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        // Draw the bar background.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..renderer::Quad::default()
            },
            nav_style.background,
        );

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
                    ..nav_style.active_indicator_color
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: cell_bounds,
                        ..renderer::Quad::default()
                    },
                    Background::Color(hover_color),
                );
            }

            // Active indicator pill (centered in cell, in the upper portion).
            if is_active {
                let pill_x = cell_bounds.x + (cell_bounds.width - INDICATOR_WIDTH) / 2.0;
                let pill_y = cell_bounds.y + 12.0;
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
                    Background::Color(nav_style.active_indicator_color),
                );
            }

            // Icon color (applied via renderer style).
            let icon_color = if is_active {
                nav_style.active_icon_color
            } else {
                nav_style.inactive_icon_color
            };

            // Draw the icon element if present.
            if let Some(icon_el) = &dest.icon
                && let Some(icon_layout) = cell_layout.children().next()
            {
                let icon_style = renderer::Style {
                    text_color: icon_color,
                };
                icon_el.as_widget().draw(
                    &tree.children[i],
                    renderer,
                    theme,
                    &icon_style,
                    icon_layout,
                    cursor,
                    viewport,
                );
            }

            // Label text — centered in cell, below the indicator area.
            let label_y = cell_bounds.y + INDICATOR_HEIGHT + 16.0;
            let label_height = cell_bounds.height - INDICATOR_HEIGHT - 16.0;

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

            let text_color = if is_active {
                nav_style.active_icon_color
            } else {
                nav_style.inactive_icon_color
            };

            renderer.fill_text(
                text,
                iced::Point::new(cell_bounds.center_x(), label_y),
                text_color,
                cell_bounds,
            );
        }

        // Suppress unused variable warning.
        let _ = style;
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
        // Icons are non-interactive leaf elements.
    }
}

impl<'a, Message, Theme, Renderer> From<NavigationBar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn from(bar: NavigationBar<'a, Message, Theme, Renderer>) -> Self {
        Element::new(bar)
    }
}
