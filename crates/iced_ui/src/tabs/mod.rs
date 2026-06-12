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
use iced::advanced::text::LineHeight;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use std::cell::Cell;
use std::marker::PhantomData;

use iced::{Background, Color, Element, Event, Length, Pixels, Rectangle, Size};

use crate::{FontSizeBase, Space, SpacingBase};

/// Vertical padding above and below a tab cell's content.
const VERTICAL_PADDING: Space = Space::sx(1.0);

/// Gap between the icon and the label inside a tab cell.
const ICON_LABEL_GAP: Space = Space::sx(0.5);

/// Thickness of the active indicator line in logical pixels.
///
/// A stroke thickness rather than a spacing value, so it stays
/// absolute and does not scale with the theme's spacing base.
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
#[derive(Debug, Clone)]
struct State {
    tabs: Vec<TabState>,
    /// Cached theme spacing base. Refreshed in [`Widget::draw`] each
    /// frame so that [`Widget::layout`] can resolve spacing tokens
    /// even though the theme is not available there.
    spacing: Cell<u8>,
    /// Cached theme base text size, refreshed alongside `spacing`.
    text_size: Cell<f32>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            spacing: Cell::new(crate::Theme::DEFAULT_SPACING),
            text_size: Cell::new(crate::Theme::DEFAULT_TEXT_SIZE),
        }
    }
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

    /// Whether any tab has an icon (affects row height).
    fn has_any_icon(&self) -> bool {
        self.tabs.iter().any(|t| t.icon.is_some())
    }
}

/// The height of a single line of label text at the given base text
/// size, using the default line height.
fn label_line_height(text_size: f32) -> f32 {
    LineHeight::default().to_absolute(Pixels(text_size)).0
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tabs<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + FontSizeBase + SpacingBase + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            tabs: vec![TabState::default(); self.tabs.len()],
            ..State::default()
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
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let (spacing, text_size) = {
            let state = tree.state.downcast_ref::<State>();
            (state.spacing.get(), state.text_size.get())
        };
        let padding = VERTICAL_PADDING.resolve(spacing);
        let gap = ICON_LABEL_GAP.resolve(spacing);
        let label_height = label_line_height(text_size);

        let tab_count = self.tabs.len();
        if tab_count == 0 {
            return layout::Node::new(Size::new(0.0, padding + label_height + padding));
        }

        let total_width = limits.resolve(self.width, Length::Shrink, Size::ZERO).width;
        let cell_width = total_width / tab_count as f32;

        // First pass: lay out icons at their natural size and find
        // the tallest one, so every cell shares a uniform icon block.
        let mut icon_nodes: Vec<Option<layout::Node>> = Vec::with_capacity(tab_count);
        let mut max_icon_height: f32 = 0.0;
        for i in 0..tab_count {
            let icon_node = if let Some(icon) = &mut self.tabs[i].icon {
                let icon_limits =
                    layout::Limits::new(Size::ZERO, Size::new(cell_width, f32::INFINITY));
                let node =
                    icon.as_widget_mut()
                        .layout(&mut tree.children[i], renderer, &icon_limits);
                max_icon_height = max_icon_height.max(node.size().height);
                Some(node)
            } else {
                None
            };
            icon_nodes.push(icon_node);
        }

        // The row height is determined by the content: vertical
        // padding around the label, plus the icon block and gap when
        // any tab carries an icon.
        let row_height = if self.has_any_icon() {
            padding + max_icon_height + gap + label_height + padding
        } else {
            padding + label_height + padding
        };

        // Second pass: position the icon blocks within their cells.
        let mut children = Vec::with_capacity(tab_count);
        for (i, icon_node) in icon_nodes.into_iter().enumerate() {
            let mut cell_children = Vec::new();

            // Center the icon horizontally and within the shared icon
            // block vertically.
            if let Some(mut icon_node) = icon_node {
                let icon_w = icon_node.size().width;
                let icon_h = icon_node.size().height;
                let icon_x = (cell_width - icon_w) / 2.0;
                let icon_y = padding + (max_icon_height - icon_h) / 2.0;
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

        // Cache the theme metrics for use in layout() on subsequent
        // frames.
        state.spacing.set(theme.spacing());
        state.text_size.set(theme.text_size());

        let padding = VERTICAL_PADDING.resolve(theme.spacing());
        let label_height = label_line_height(theme.text_size());

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

            // Text label — bottom-anchored: it sits just above the
            // bottom padding, both with and without an icon block
            // above it.
            let text_y = cell_bounds.y + cell_bounds.height - padding - label_height / 2.0;

            let text = iced::advanced::text::Text {
                content: tab.label.clone(),
                bounds: Size::new(cell_bounds.width, label_height),
                size: Pixels(theme.text_size()),
                line_height: LineHeight::default(),
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
    Theme: Catalog + FontSizeBase + SpacingBase + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn from(tabs: Tabs<'a, Message, Theme, Renderer>) -> Self {
        Element::new(tabs)
    }
}
