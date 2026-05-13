//! Horizontal tab row with an active indicator.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::tabs::{Tab, Tabs};
//!
//! let tabs = Tabs::new(Message::TabSelected)
//!     .push(Tab::new("Day"))
//!     .push(Tab::new("Week"))
//!     .push(Tab::new("Month"))
//!     .active(selected_index);
//! ```

mod style;

pub use style::{Catalog, Style, StyleFn, TabStatus, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use std::marker::PhantomData;

use iced::{Background, Color, Element, Event, Length, Rectangle, Size};

/// Height of a tab cell in logical pixels (MD3 spec).
const TAB_HEIGHT: f32 = 48.0;

/// Thickness of the active indicator line in logical pixels.
const INDICATOR_HEIGHT: f32 = 3.0;

/// A single tab within a [`Tabs`] widget.
///
/// Contains a text label.
pub struct Tab {
    label: String,
}

impl Tab {
    /// Creates a new tab with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

/// Internal per-tab interaction state.
#[derive(Debug, Clone, Default)]
struct TabState {
    is_hovered: bool,
}

/// Internal state for the whole tab row.
#[derive(Debug, Clone, Default)]
struct State {
    tabs: Vec<TabState>,
}

/// Renders as a horizontal sequence of equal-width cells. The active
/// tab is highlighted with an indicator line at the bottom.
pub struct Tabs<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    tabs: Vec<Tab>,
    active: usize,
    on_select: Box<dyn Fn(usize) -> Message + 'a>,
    class: Theme::Class<'a>,
    width: Length,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer> Tabs<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    /// Creates a new [`Tabs`] widget with the given selection callback.
    ///
    /// The callback receives the zero-based index of the selected tab.
    pub fn new(on_select: impl Fn(usize) -> Message + 'a) -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            on_select: Box::new(on_select),
            class: Theme::default(),
            width: Length::Fill,
            _renderer: PhantomData,
        }
    }

    /// Appends a [`Tab`] to the row.
    pub fn push(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }

    /// Sets the index of the currently active tab.
    pub fn active(mut self, index: usize) -> Self {
        self.active = index;
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the width of the tab row.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tabs<'a, Message, Theme, Renderer>
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
            tabs: vec![TabState::default(); self.tabs.len()],
        })
    }

    fn children(&self) -> Vec<Tree> {
        Vec::new()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        state.tabs.resize_with(self.tabs.len(), Default::default);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(TAB_HEIGHT))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let tab_count = self.tabs.len();
        if tab_count == 0 {
            return layout::Node::new(Size::new(0.0, TAB_HEIGHT));
        }

        let total_width = limits.resolve(self.width, Length::Shrink, Size::ZERO).width;
        let cell_width = total_width / tab_count as f32;

        let mut children = Vec::with_capacity(tab_count);
        for i in 0..tab_count {
            let mut node = layout::Node::new(Size::new(cell_width, TAB_HEIGHT));
            node = node.move_to(iced::Point::new(cell_width * i as f32, 0.0));
            children.push(node);
        }

        layout::Node::with_children(Size::new(total_width, TAB_HEIGHT), children)
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

        // Draw the row background if set.
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

        // Draw each tab cell.
        for (i, (tab, cell_layout)) in self.tabs.iter().zip(layout.children()).enumerate() {
            let cell_bounds = cell_layout.bounds();
            let is_active = i == self.active;
            let tab_state = state.tabs.get(i).cloned().unwrap_or_default();

            let status = if is_active {
                TabStatus::Active
            } else if tab_state.is_hovered {
                TabStatus::Hovered
            } else {
                TabStatus::Inactive
            };

            // Hover overlay for inactive/hovered tabs.
            if status == TabStatus::Hovered {
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

            // Text label — centered in cell.
            let text_color = if is_active {
                style.active_text_color
            } else {
                style.inactive_text_color
            };

            let text = iced::advanced::text::Text {
                content: tab.label.clone(),
                bounds: Size::new(cell_bounds.width, cell_bounds.height),
                size: renderer.default_size(),
                line_height: iced::advanced::text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                shaping: iced::advanced::text::Shaping::Advanced,
                wrapping: iced::advanced::text::Wrapping::None,
            };

            renderer.fill_text(
                text,
                iced::Point::new(cell_bounds.center_x(), cell_bounds.center_y()),
                text_color,
                cell_bounds,
            );

            // Active indicator line at the bottom.
            if is_active {
                let indicator_bounds = Rectangle {
                    x: cell_bounds.x,
                    y: cell_bounds.y + cell_bounds.height - INDICATOR_HEIGHT,
                    width: cell_bounds.width,
                    height: INDICATOR_HEIGHT,
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: indicator_bounds,
                        ..renderer::Quad::default()
                    },
                    Background::Color(style.active_indicator_color),
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
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        state.tabs.resize_with(self.tabs.len(), Default::default);

        for (i, cell_layout) in layout.children().enumerate() {
            let bounds = cell_layout.bounds();
            let is_over = cursor.is_over(bounds);

            if let Some(tab_state) = state.tabs.get_mut(i) {
                match event {
                    Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                        tab_state.is_hovered = is_over;
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

impl<'a, Message, Theme, Renderer> From<Tabs<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn from(tabs: Tabs<'a, Message, Theme, Renderer>) -> Self {
        Element::new(tabs)
    }
}
