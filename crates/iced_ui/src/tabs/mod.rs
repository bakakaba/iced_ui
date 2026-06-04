//! Horizontal tab row with an active indicator.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::tabs::{Tab, Tabs};
//!
//! let tabs = Tabs::new(Message::TabSelected)
//!     .push(Tab::new("Day").icon(text("D").font(MY_ICON_FONT)))
//!     .push(Tab::new("Week").icon(text("W").font(MY_ICON_FONT)))
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

use iced::{Background, Color, Element, Event, Length, Pixels, Rectangle, Size};

use crate::FontSizeBase;

/// Height of a tab cell in logical pixels (label-only).
const TAB_HEIGHT: f32 = 48.0;

/// Height of a tab cell when an icon is present (MD3 primary tabs).
const TAB_HEIGHT_WITH_ICON: f32 = 64.0;

/// Size allocated for the icon inside a tab cell.
const ICON_SIZE: f32 = 24.0;

/// Thickness of the active indicator line in logical pixels.
const INDICATOR_HEIGHT: f32 = 3.0;

/// A single tab within a [`Tabs`] widget.
///
/// Contains a text label and an optional icon element.
pub struct Tab<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer> {
    label: String,
    icon: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Tab<'a, Message, Theme, Renderer> {
    /// Creates a new tab with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    /// Sets the icon element for this tab.
    ///
    /// The icon is rendered centered above the text label. Any iced
    /// widget can be used — typically a `text()` glyph from an icon
    /// font. When any tab has an icon, the tab row height increases
    /// to accommodate the icon.
    pub fn icon(mut self, icon: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.icon = Some(icon.into());
        self
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
    tabs: Vec<Tab<'a, Message, Theme, Renderer>>,
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
    pub fn push(mut self, tab: Tab<'a, Message, Theme, Renderer>) -> Self {
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

    /// Whether any tab has an icon (determines row height).
    fn has_any_icon(&self) -> bool {
        self.tabs.iter().any(|t| t.icon.is_some())
    }

    /// The effective row height based on icon presence.
    fn row_height(&self) -> f32 {
        if self.has_any_icon() {
            TAB_HEIGHT_WITH_ICON
        } else {
            TAB_HEIGHT
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tabs<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + FontSizeBase + 'a,
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
        self.tabs
            .iter()
            .map(|tab| match &tab.icon {
                Some(icon) => Tree::new(icon),
                None => Tree::empty(),
            })
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        state.tabs.resize_with(self.tabs.len(), Default::default);

        // Diff icon children.
        tree.children.resize_with(self.tabs.len(), Tree::empty);
        for (i, tab) in self.tabs.iter().enumerate() {
            if let Some(icon) = &tab.icon {
                tree.children[i].diff(icon);
            }
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(self.row_height()))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let tab_count = self.tabs.len();
        if tab_count == 0 {
            return layout::Node::new(Size::new(0.0, self.row_height()));
        }

        let row_height = self.row_height();
        let total_width = limits.resolve(self.width, Length::Shrink, Size::ZERO).width;
        let cell_width = total_width / tab_count as f32;

        let mut children = Vec::with_capacity(tab_count);
        for i in 0..tab_count {
            // Layout the icon if present.
            let icon_child = if let Some(icon) = &mut self.tabs[i].icon {
                let icon_limits = layout::Limits::new(Size::ZERO, Size::new(ICON_SIZE, ICON_SIZE));
                let icon_node =
                    icon.as_widget_mut()
                        .layout(&mut tree.children[i], renderer, &icon_limits);
                Some(icon_node)
            } else {
                None
            };

            let mut cell_children = Vec::new();

            // Position the icon centered horizontally, in the upper portion.
            if let Some(mut icon_node) = icon_child {
                let icon_w = icon_node.size().width;
                let icon_h = icon_node.size().height;
                let icon_x = (cell_width - icon_w) / 2.0;
                // Place icon at ~12px from top of cell.
                let icon_y = 12.0 + (ICON_SIZE - icon_h) / 2.0;
                icon_node = icon_node.move_to(iced::Point::new(icon_x, icon_y));
                cell_children.push(icon_node);
            }

            let mut cell_node =
                layout::Node::with_children(Size::new(cell_width, row_height), cell_children);
            cell_node = cell_node.move_to(iced::Point::new(cell_width * i as f32, 0.0));
            children.push(cell_node);
        }

        layout::Node::with_children(Size::new(total_width, row_height), children)
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
        let tab_style = Catalog::style(theme, &self.class);
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let has_icons = self.has_any_icon();

        // Draw the row background if set.
        if let Some(bg) = tab_style.background {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: tab_style.border,
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
                    ..tab_style.active_indicator_color
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: cell_bounds,
                        ..renderer::Quad::default()
                    },
                    Background::Color(hover_color),
                );
            }

            // Text and icon color.
            let text_color = if is_active {
                tab_style.active_text_color
            } else {
                tab_style.inactive_text_color
            };

            // Draw the icon element if present.
            if let Some(icon_el) = &tab.icon
                && let Some(icon_layout) = cell_layout.children().next()
            {
                let icon_style = renderer::Style { text_color };
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

            // Text label — centered in cell. When icons are present,
            // the label sits in the lower portion.
            let (text_y, text_bounds_height) = if has_icons {
                // Icon occupies top ~36px, label occupies the rest
                let label_top = 12.0 + ICON_SIZE + 4.0; // 40px from top
                let label_h = cell_bounds.height - label_top - INDICATOR_HEIGHT;
                (cell_bounds.y + label_top + label_h / 2.0, label_h)
            } else {
                (cell_bounds.center_y(), cell_bounds.height)
            };

            let text = iced::advanced::text::Text {
                content: tab.label.clone(),
                bounds: Size::new(cell_bounds.width, text_bounds_height),
                size: Pixels(theme.text_size()),
                line_height: iced::advanced::text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                shaping: iced::advanced::text::Shaping::Advanced,
                wrapping: iced::advanced::text::Wrapping::None,
            };

            renderer.fill_text(
                text,
                iced::Point::new(cell_bounds.center_x(), text_y),
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
                    Background::Color(tab_style.active_indicator_color),
                );
            }
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
        // Icons are non-interactive leaf elements.
    }
}

impl<'a, Message, Theme, Renderer> From<Tabs<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + FontSizeBase + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn from(tabs: Tabs<'a, Message, Theme, Renderer>) -> Self {
        Element::new(tabs)
    }
}
