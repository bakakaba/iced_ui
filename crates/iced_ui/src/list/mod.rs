//! A vertical list of interactive items with hover/press feedback.
//!
//! Each item wraps an arbitrary [`Element`] and optionally emits a
//! message when pressed. The widget does **not** maintain internal
//! selection state — the parent application is responsible for
//! tracking which item is "active" and communicating that through
//! styling.
//!
//! See [`List`] for the widget and [`Catalog`]/[`ItemStyle`] for
//! styling.

mod style;

use std::cell::Cell;
use std::marker::PhantomData;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Padding, Rectangle, Size, Vector};

pub use style::{Catalog, ItemStyle, Status, StyleFn, default};

use crate::{PaddingSource, Space, SpacingBase};

/// A single entry in a [`List`].
///
/// Wraps an arbitrary [`Element`] and optionally emits a `Message`
/// when pressed.
pub struct Item<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<Message>,
}

impl<'a, Message, Theme, Renderer> Item<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new [`Item`] wrapping the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            on_press: None,
        }
    }

    /// Sets the message emitted when this item is pressed.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the message emitted when this item is pressed, if `Some`.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }
}

/// Internal per-item interaction state.
#[derive(Debug, Clone, Copy, Default)]
struct ItemState {
    is_hovered: bool,
    is_pressed: bool,
}

/// Internal widget state for a [`List`].
#[derive(Debug)]
struct ListState {
    items: Vec<ItemState>,
    /// Cached padding resolution for layout.
    padding_cache: Cell<Padding>,
    /// Cached item-padding resolution for layout.
    item_padding_cache: Cell<Padding>,
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            padding_cache: Cell::new(Padding::new(0.0)),
            item_padding_cache: Cell::new(Padding::new(0.0)),
        }
    }
}

/// A vertical list of interactive items.
///
/// Each item can contain arbitrary content and optionally emit a
/// message when pressed. The list provides hover and press visual
/// feedback via its [`Catalog`] style, but does **not** track
/// selection internally.
///
/// # Example
///
/// ```no_run
/// use iced::widget::text;
/// use iced_ui::{list, Theme};
///
/// # type Message = ();
/// # fn _build() -> iced::Element<'static, Message, Theme> {
/// list::List::new()
///     .push(list::Item::new(text("First")))
///     .push(list::Item::new(text("Second")))
///     .into()
/// # }
/// ```
pub struct List<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    items: Vec<Item<'a, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
    padding: PaddingSource,
    item_padding: PaddingSource,
    spacing: Space,
    class: <Theme as Catalog>::Class<'a>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer> Default for List<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> List<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new, empty [`List`].
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            width: Length::Fill,
            height: Length::Shrink,
            padding: PaddingSource::from(Space::sx(1.0)),
            item_padding: PaddingSource::from(Space::sx(1.0)),
            spacing: Space::sx(0.0),
            class: <Theme as Catalog>::default(),
            _renderer: PhantomData,
        }
    }

    /// Appends an [`Item`] to the list.
    pub fn push(mut self, item: Item<'a, Message, Theme, Renderer>) -> Self {
        self.items.push(item);
        self
    }

    /// Sets the width of the [`List`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`List`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the outer padding of the [`List`]. Defaults to
    /// [`Space::sx(1.0)`](Space::sx).
    pub fn padding(mut self, padding: impl Into<PaddingSource>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the inner padding of each item row. Defaults to
    /// [`Space::sx(1.0)`](Space::sx).
    pub fn item_padding(mut self, padding: impl Into<PaddingSource>) -> Self {
        self.item_padding = padding.into();
        self
    }

    /// Sets the vertical spacing between items. Defaults to no
    /// spacing.
    pub fn spacing(mut self, spacing: impl Into<Space>) -> Self {
        self.spacing = spacing.into();
        self
    }
}

impl<'a, Message, Renderer> List<'a, Message, crate::Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Sets the style of the [`List`] using a
    /// `Fn(&Theme, Status) -> ItemStyle` closure.
    pub fn style(mut self, style: impl Fn(&crate::Theme, Status) -> ItemStyle + 'a) -> Self {
        self.class = Box::new(style);
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for List<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog + SpacingBase,
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ListState>()
    }

    fn state(&self) -> tree::State {
        let initial_padding = self.padding.resolve(crate::Theme::DEFAULT_SPACING);
        let initial_item_padding = self.item_padding.resolve(crate::Theme::DEFAULT_SPACING);
        tree::State::new(ListState {
            items: vec![ItemState::default(); self.items.len()],
            padding_cache: Cell::new(initial_padding),
            item_padding_cache: Cell::new(initial_item_padding),
        })
    }

    fn children(&self) -> Vec<Tree> {
        self.items
            .iter()
            .map(|item| Tree::new(&item.content))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let elements: Vec<&Element<'a, Message, Theme, Renderer>> =
            self.items.iter().map(|item| &item.content).collect();
        tree.diff_children(&elements);

        // Ensure the item-state vec matches the item count.
        let state = tree.state.downcast_mut::<ListState>();
        state
            .items
            .resize_with(self.items.len(), ItemState::default);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_ref::<ListState>();
        let padding = state.padding_cache.get();
        let item_padding = state.item_padding_cache.get();
        // Resolve spacing using default base for layout (will be refreshed in draw).
        let spacing = self.spacing.resolve(crate::Theme::DEFAULT_SPACING);

        let limits = limits.width(self.width).height(self.height).shrink(padding);

        let item_limits = limits.width(Length::Fill);
        let available_width = limits
            .resolve(Length::Fill, Length::Shrink, Size::ZERO)
            .width;

        let mut children = Vec::with_capacity(self.items.len());
        let mut total_height: f32 = 0.0;

        for (i, item) in self.items.iter_mut().enumerate() {
            if i > 0 {
                total_height += spacing;
            }

            let item_inner_limits = item_limits.shrink(item_padding);
            let content_node = item.content.as_widget_mut().layout(
                &mut tree.children[i],
                renderer,
                &item_inner_limits,
            );

            let content_size = content_node.size();
            let item_size = Size::new(
                available_width,
                content_size.height + item_padding.top + item_padding.bottom,
            );

            let mut item_node = layout::Node::with_children(
                item_size,
                vec![content_node.move_to((item_padding.left, item_padding.top))],
            );
            item_node = item_node.move_to((padding.left, (padding.top + total_height)));

            total_height += item_size.height;
            children.push(item_node);
        }

        let total_size = Size::new(
            available_width + padding.left + padding.right,
            total_height + padding.top + padding.bottom,
        );

        let resolved = limits.resolve(self.width, self.height, total_size);
        layout::Node::with_children(resolved.max(total_size), children)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        for ((item, child_tree), child_layout) in self
            .items
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            let content_layout = child_layout.children().next().unwrap();
            item.content
                .as_widget_mut()
                .operate(child_tree, content_layout, renderer, operation);
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
        let state = tree.state.downcast_mut::<ListState>();
        state
            .items
            .resize_with(self.items.len(), ItemState::default);

        // First, propagate events to child content.
        for ((item, child_tree), child_layout) in self
            .items
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            let content_layout = child_layout.children().next().unwrap();
            item.content.as_widget_mut().update(
                child_tree,
                event,
                content_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        // Now handle item-level interactions.
        for (i, child_layout) in layout.children().enumerate() {
            let bounds = child_layout.bounds();
            let is_over = cursor.is_over(bounds);

            let item_state = &mut state.items[i];
            item_state.is_hovered = is_over;

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if is_over => {
                    item_state.is_pressed = true;
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    if item_state.is_pressed
                        && is_over
                        && let Some(msg) = &self.items[i].on_press
                    {
                        shell.publish(msg.clone());
                    }
                    item_state.is_pressed = false;
                }
                _ => {}
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        // Check children first.
        for ((item, child_tree), child_layout) in self
            .items
            .iter()
            .zip(tree.children.iter())
            .zip(layout.children())
        {
            let content_layout = child_layout.children().next().unwrap();
            let interaction = item.content.as_widget().mouse_interaction(
                child_tree,
                content_layout,
                cursor,
                viewport,
                renderer,
            );
            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }

        // If hovering a clickable item, show pointer.
        for (i, child_layout) in layout.children().enumerate() {
            if cursor.is_over(child_layout.bounds()) && self.items[i].on_press.is_some() {
                return mouse::Interaction::Pointer;
            }
        }

        mouse::Interaction::None
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<ListState>();

        // Refresh padding caches.
        state
            .padding_cache
            .set(self.padding.resolve(theme.spacing()));
        state
            .item_padding_cache
            .set(self.item_padding.resolve(theme.spacing()));

        for (i, child_layout) in layout.children().enumerate() {
            let bounds = child_layout.bounds();

            if bounds.intersection(viewport).is_none() {
                continue;
            }

            let item_state = state.items.get(i).copied().unwrap_or_default();
            let status = if item_state.is_pressed {
                Status::Pressed
            } else if item_state.is_hovered && self.items[i].on_press.is_some() {
                Status::Hovered
            } else {
                Status::Active
            };

            let style = theme.style(&self.class, status);

            // Draw item background.
            if let Some(background) = style.background {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border: style.border,
                        ..renderer::Quad::default()
                    },
                    background,
                );
            }

            // Draw item content.
            let content_layout = child_layout.children().next().unwrap();
            let item_renderer_style = if let Some(color) = style.text_color {
                renderer::Style { text_color: color }
            } else {
                *renderer_style
            };

            self.items[i].content.as_widget().draw(
                &tree.children[i],
                renderer,
                theme,
                &item_renderer_style,
                content_layout,
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        for ((item, child_tree), child_layout) in self
            .items
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            let content_layout = child_layout.children().next().unwrap();
            if let Some(overlay) = item.content.as_widget_mut().overlay(
                child_tree,
                content_layout,
                renderer,
                viewport,
                translation,
            ) {
                return Some(overlay);
            }
        }
        None
    }
}

impl<'a, Message, Theme, Renderer> From<List<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(list: List<'a, Message, Theme, Renderer>) -> Self {
        Self::new(list)
    }
}
