//! A standalone menu trigger that opens a floating dropdown popup.
//!
//! Unlike [`MenuBar`](super::MenuBar), which manages a row of
//! top-level menus with inter-menu coordination, `MenuButton` wraps
//! a single trigger element and opens a single dropdown when clicked.
//!
//! # Example
//!
//! ```ignore
//! use iced::widget::text;
//! use iced_ui::button::{Button, Variant};
//! use iced_ui::menu::{MenuButton, Item, Separator};
//!
//! let menu = MenuButton::new(
//!     Button::new(text("Actions")).variant(Variant::Ghost),
//!     vec![
//!         Item::new("Cut").on_press(Msg::Cut).into(),
//!         Item::new("Copy").on_press(Msg::Copy).into(),
//!         Separator.into(),
//!         Item::new("Paste").on_press(Msg::Paste).into(),
//!     ],
//! );
//! ```

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay as advanced_overlay;
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{
    Element, Event, Font, Length, Padding, Pixels, Point, Rectangle, Size, Vector, keyboard,
};

use super::item::{Entry, Menu};
use super::overlay::{Metrics, draw_menu, hit_row, layout_menu};
use super::style::{Catalog, Style};

use crate::SpacingBase;

/// Internal widget state.
#[derive(Debug, Default)]
struct State<P: Paragraph> {
    /// Whether the popup is open.
    open: bool,
    /// Path of open submenus (indices into nested entries). Empty when
    /// only the top-level popup is showing.
    open_path: Vec<usize>,
    /// Hovered row per depth.
    hover_path: Vec<Option<usize>>,
    /// Cached paragraph (unused directly but needed for type parity).
    _paragraph: std::marker::PhantomData<P>,
}

/// A standalone menu trigger widget.
///
/// Wraps any [`Element`] as a trigger. When the trigger is clicked, a
/// floating dropdown popup appears below it, containing the provided
/// menu entries. Clicking an item publishes its message and closes
/// the popup.
pub struct MenuButton<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    trigger: Element<'a, Message, Theme, Renderer>,
    entries: Vec<Entry<Message>>,
    text_size: Option<Pixels>,
    font: Option<Font>,
    class: <Theme as Catalog>::Class<'a>,
}

impl<'a, Message, Theme, Renderer> MenuButton<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    /// Creates a new `MenuButton` with the given trigger element and
    /// menu entries.
    pub fn new(
        trigger: impl Into<Element<'a, Message, Theme, Renderer>>,
        entries: Vec<Entry<Message>>,
    ) -> Self {
        Self {
            trigger: trigger.into(),
            entries,
            text_size: None,
            font: None,
            class: <Theme as Catalog>::default(),
        }
    }

    /// Sets the text size used for menu items in the dropdown.
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = Some(size.into());
        self
    }

    /// Sets the font used for menu items in the dropdown.
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }
}

impl<'a, Message, Renderer> MenuButton<'a, Message, crate::Theme, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer<Font = Font>,
{
    /// Sets the style of the dropdown popup.
    pub fn style(mut self, style: impl Fn(&crate::Theme) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for MenuButton<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + 'a,
    Renderer: text::Renderer<Font = Font> + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph> {
            open: false,
            open_path: Vec::new(),
            hover_path: Vec::new(),
            _paragraph: std::marker::PhantomData,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.trigger)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.trigger]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.trigger
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
        self.trigger.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
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
        // Forward to trigger child.
        self.trigger.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();

        // Toggle open/close on click.
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && let Some(pos) = cursor.position()
            && bounds.contains(pos)
        {
            state.open = !state.open;
            if state.open {
                state.open_path.clear();
                state.hover_path.clear();
            }
            shell.capture_event();
            shell.request_redraw();
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
        self.trigger.as_widget().mouse_interaction(
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
        self.trigger
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<advanced_overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        if !state.open {
            return None;
        }

        let trigger_bounds = layout.bounds() + translation;
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size()).0;
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let padding = Padding::new(4.0);
        let spacing = 8.0;
        let metrics = Metrics::new(text_size, padding, spacing);

        let overlay = MenuButtonOverlay {
            entries: &mut self.entries,
            state,
            trigger_bounds,
            metrics,
            font,
            style_fn: &self.class,
        };

        Some(advanced_overlay::Element::new(Box::new(overlay)))
    }
}

impl<'a, Message, Theme, Renderer> From<MenuButton<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + 'a,
    Renderer: text::Renderer<Font = Font> + 'a,
{
    fn from(menu_button: MenuButton<'a, Message, Theme, Renderer>) -> Self {
        Element::new(menu_button)
    }
}

// --- Overlay ---

struct MenuButtonOverlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    entries: &'b mut [Entry<Message>],
    state: &'b mut State<Renderer::Paragraph>,
    trigger_bounds: Rectangle,
    metrics: Metrics,
    font: Font,
    style_fn: &'b <Theme as Catalog>::Class<'a>,
}

impl<Message, Theme, Renderer> advanced_overlay::Overlay<Message, Theme, Renderer>
    for MenuButtonOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        // Build a temporary Menu to feed into layout_menu.
        let menu = self.as_menu();
        let anchor = Point::new(
            self.trigger_bounds.x,
            self.trigger_bounds.y + self.trigger_bounds.height,
        );

        let mut nodes = Vec::new();

        // Top-level popup.
        let top_node = layout_menu::<Message, Renderer>(
            &menu,
            renderer,
            self.font,
            self.metrics,
            anchor,
            bounds,
        );
        nodes.push(top_node);

        // Submenu panels along open_path.
        let mut current_entries: &[Entry<Message>] = self.entries;
        let mut prev_node_bounds = nodes[0].bounds();
        for &idx in &self.state.open_path {
            if let Some(Entry::Submenu(sub)) = current_entries.get(idx) {
                let row_top = prev_node_bounds.y
                    + self.metrics.popup_padding.top
                    + row_offset_entries(current_entries, idx, self.metrics);
                let sub_anchor = Point::new(prev_node_bounds.x + prev_node_bounds.width, row_top);
                let sub_node = layout_menu::<Message, Renderer>(
                    sub,
                    renderer,
                    self.font,
                    self.metrics,
                    sub_anchor,
                    bounds,
                );
                prev_node_bounds = sub_node.bounds();
                nodes.push(sub_node);
                current_entries = &sub.entries;
            } else {
                break;
            }
        }

        let total_bounds = nodes
            .iter()
            .map(layout::Node::bounds)
            .fold::<Option<Rectangle>, _>(None, |acc, b| match acc {
                Some(a) => Some(a.union(&b)),
                None => Some(b),
            })
            .unwrap_or(Rectangle::with_size(Size::ZERO));

        let nodes: Vec<layout::Node> = nodes
            .into_iter()
            .map(|node| {
                let b = node.bounds();
                node.move_to(Point::new(b.x - total_bounds.x, b.y - total_bounds.y))
            })
            .collect();

        layout::Node::with_children(total_bounds.size(), nodes)
            .move_to(Point::new(total_bounds.x, total_bounds.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let style = theme.style(self.style_fn);

        let mut current_entries: &[Entry<Message>] = self.entries;
        for (depth, child_layout) in layout.children().enumerate() {
            let menu = entries_to_menu(current_entries);
            let hovered = self.state.hover_path.get(depth).copied().flatten();
            let opened = self.state.open_path.get(depth).copied();

            draw_menu::<Message, Renderer>(
                renderer,
                &menu,
                child_layout,
                mouse::Cursor::Unavailable,
                &style,
                self.metrics,
                self.font,
                hovered,
                opened,
            );

            // Descend into the opened submenu for the next depth.
            if let Some(idx) = self.state.open_path.get(depth).copied() {
                if let Some(Entry::Submenu(sub)) = current_entries.get(idx) {
                    current_entries = &sub.entries;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let child_layouts: Vec<Layout<'_>> = layout.children().collect();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let Some(pos) = cursor.position() else {
                    return;
                };

                // Update hover_path: find which row is hovered at each depth.
                let mut new_hover: Vec<Option<usize>> = Vec::new();
                let mut current_entries: &[Entry<Message>] = self.entries;

                for (depth, child_layout) in child_layouts.iter().enumerate() {
                    let bounds = child_layout.bounds();
                    let row = if bounds.contains(pos) {
                        let menu = entries_to_menu(current_entries);
                        hit_row(&menu, *child_layout, self.metrics, pos)
                    } else {
                        None
                    };
                    new_hover.push(row);

                    // Check if we should open/close a submenu.
                    if let Some(idx) = row
                        && let Some(Entry::Submenu(sub)) = current_entries.get(idx)
                    {
                        // Hovering a submenu entry: extend open_path.
                        if self.state.open_path.get(depth) != Some(&idx) {
                            self.state.open_path.truncate(depth);
                            self.state.open_path.push(idx);
                            shell.request_redraw();
                        }
                        current_entries = &sub.entries;
                        continue;
                    }

                    // If not hovering a submenu but there's an open path
                    // at this depth, continue descending — the cursor may
                    // be in a deeper panel.
                    if let Some(&idx) = self.state.open_path.get(depth)
                        && let Some(Entry::Submenu(sub)) = current_entries.get(idx)
                    {
                        // If cursor is in this panel on a non-submenu
                        // row, close deeper panels.
                        if bounds.contains(pos) {
                            self.state.open_path.truncate(depth);
                            shell.request_redraw();
                            break;
                        }
                        current_entries = &sub.entries;
                        continue;
                    }

                    break;
                }

                if new_hover != self.state.hover_path {
                    self.state.hover_path = new_hover;
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(pos) = cursor.position() else {
                    return;
                };

                // Check if click is inside any open panel.
                let mut current_entries: &[Entry<Message>] = self.entries;
                for (depth, child_layout) in child_layouts.iter().enumerate() {
                    let bounds = child_layout.bounds();
                    if bounds.contains(pos) {
                        let menu = entries_to_menu(current_entries);
                        if let Some(row) = hit_row(&menu, *child_layout, self.metrics, pos) {
                            match current_entries.get(row) {
                                Some(Entry::Item(item)) if item.is_activatable() => {
                                    if let Some(msg) = &item.on_press {
                                        shell.publish(msg.clone());
                                    }
                                    self.state.open = false;
                                    self.state.open_path.clear();
                                    self.state.hover_path.clear();
                                    shell.capture_event();
                                    shell.request_redraw();
                                    return;
                                }
                                Some(Entry::Submenu(_)) => {
                                    // Clicking a submenu entry opens it.
                                    self.state.open_path.truncate(depth);
                                    self.state.open_path.push(row);
                                    shell.capture_event();
                                    shell.request_redraw();
                                    return;
                                }
                                _ => {}
                            }
                        }
                        shell.capture_event();
                        return;
                    }

                    if let Some(idx) = self.state.open_path.get(depth).copied() {
                        if let Some(Entry::Submenu(sub)) = current_entries.get(idx) {
                            current_entries = &sub.entries;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // Click outside all panels — close.
                if !self.trigger_bounds.contains(pos) {
                    self.state.open = false;
                    self.state.open_path.clear();
                    self.state.hover_path.clear();
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                self.state.open = false;
                self.state.open_path.clear();
                self.state.hover_path.clear();
                shell.capture_event();
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(pos) = cursor.position() else {
            return mouse::Interaction::default();
        };

        let mut current_entries: &[Entry<Message>] = self.entries;
        for (depth, child_layout) in layout.children().enumerate() {
            let bounds = child_layout.bounds();
            if bounds.contains(pos) {
                let menu = entries_to_menu(current_entries);
                if let Some(row) = hit_row(&menu, child_layout, self.metrics, pos) {
                    match current_entries.get(row) {
                        Some(Entry::Item(item)) if item.is_activatable() => {
                            return mouse::Interaction::Pointer;
                        }
                        Some(Entry::Submenu(_)) => {
                            return mouse::Interaction::Pointer;
                        }
                        _ => {}
                    }
                }
                return mouse::Interaction::default();
            }

            if let Some(idx) = self.state.open_path.get(depth).copied() {
                if let Some(Entry::Submenu(sub)) = current_entries.get(idx) {
                    current_entries = &sub.entries;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        mouse::Interaction::default()
    }
}

impl<Message, Theme, Renderer> MenuButtonOverlay<'_, '_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    fn as_menu(&self) -> Menu<Message>
    where
        Message: Clone,
    {
        entries_to_menu(self.entries)
    }
}

/// Creates a temporary `Menu` wrapper around a slice of entries for
/// use with the shared `layout_menu`/`draw_menu` helpers.
fn entries_to_menu<Message: Clone>(entries: &[Entry<Message>]) -> Menu<Message> {
    Menu {
        label: String::new(),
        entries: entries.iter().map(clone_entry).collect(),
    }
}

fn clone_entry<Message: Clone>(entry: &Entry<Message>) -> Entry<Message> {
    match entry {
        Entry::Item(item) => Entry::Item(super::item::Item {
            label: item.label.clone(),
            icon: item.icon.clone(),
            shortcut: item.shortcut.clone(),
            on_press: item.on_press.clone(),
            enabled: item.enabled,
            checked: item.checked,
        }),
        Entry::Separator => Entry::Separator,
        Entry::Submenu(sub) => Entry::Submenu(Menu {
            label: sub.label.clone(),
            entries: sub.entries.iter().map(clone_entry).collect(),
        }),
    }
}

/// Like `row_offset` but works directly on an `&[Entry]` slice.
fn row_offset_entries<Message>(entries: &[Entry<Message>], index: usize, metrics: Metrics) -> f32 {
    let mut y = 0.0;
    for (i, entry) in entries.iter().enumerate() {
        if i == index {
            break;
        }
        y += match entry {
            Entry::Separator => metrics.separator_height,
            _ => metrics.row_height,
        };
    }
    y
}
