//! Side panel with destinations, supporting modal (overlay with scrim) mode.
//!
//! The [`NavigationDrawer`] widget wraps "host" content. When
//! `expanded` is `true`, a drawer panel slides from the left. If
//! `modal` is `true`, the drawer is rendered as an overlay with a
//! scrim; clicking the scrim fires `on_dismiss`.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::navigation_drawer::{NavigationDrawer, DrawerItem};
//!
//! let drawer = NavigationDrawer::new(host_content)
//!     .push(DrawerItem::header("Mail"))
//!     .push(DrawerItem::destination("Inbox"))
//!     .push(DrawerItem::destination("Sent"))
//!     .push(DrawerItem::divider())
//!     .push(DrawerItem::destination("Trash"))
//!     .active(0)
//!     .modal(true)
//!     .expanded(self.drawer_open)
//!     .on_dismiss(Message::CloseDrawer)
//!     .on_select(Message::SelectDestination);
//! ```

mod style;

pub use style::{Catalog, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::text::{self, Text};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Border, Element, Event, Length, Pixels, Point, Rectangle, Size, Vector};

use crate::{RoundnessBase, SpacingBase};

/// Width of the navigation drawer panel (MD3 spec: 360px).
const DRAWER_WIDTH: f32 = 360.0;

/// Height of a destination row.
const DESTINATION_HEIGHT: f32 = 56.0;

/// Height of a header row.
const HEADER_HEIGHT: f32 = 56.0;

/// Height of a divider (1px line + padding).
const DIVIDER_HEIGHT: f32 = 17.0;

/// Horizontal padding inside the drawer.
const DRAWER_PADDING: f32 = 12.0;

/// An item inside the navigation drawer.
#[derive(Debug, Clone)]
pub enum DrawerItem {
    /// A navigable destination with a text label.
    Destination(String),
    /// A horizontal divider line.
    Divider,
    /// A section header with bold text.
    Header(String),
}

impl DrawerItem {
    /// Creates a destination item.
    pub fn destination(label: impl Into<String>) -> Self {
        Self::Destination(label.into())
    }

    /// Creates a divider item.
    pub fn divider() -> Self {
        Self::Divider
    }

    /// Creates a header item.
    pub fn header(text: impl Into<String>) -> Self {
        Self::Header(text.into())
    }
}

/// Wraps host content and conditionally displays a drawer panel from
/// the left side of the viewport.
pub struct NavigationDrawer<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    host: Element<'a, Message, Theme, Renderer>,
    items: Vec<DrawerItem>,
    active: Option<usize>,
    modal: bool,
    expanded: bool,
    on_dismiss: Option<Message>,
    on_select: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> NavigationDrawer<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new navigation drawer wrapping the given host content.
    pub fn new(host: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            host: host.into(),
            items: Vec::new(),
            active: None,
            modal: false,
            expanded: false,
            on_dismiss: None,
            on_select: None,
            class: Theme::default(),
        }
    }

    /// Appends a drawer item.
    pub fn push(mut self, item: DrawerItem) -> Self {
        self.items.push(item);
        self
    }

    /// Sets the active destination index (zero-based, counting only
    /// `Destination` items in order).
    pub fn active(mut self, index: usize) -> Self {
        self.active = Some(index);
        self
    }

    /// Controls whether the drawer is modal (overlay with scrim).
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Controls whether the drawer is expanded (visible).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Sets the message emitted when the scrim is pressed (modal mode).
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    /// Sets the callback for destination selection. The callback
    /// receives the zero-based destination index.
    pub fn on_select(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// Interaction state for the drawer overlay.
#[derive(Debug, Default)]
struct DrawerState {
    scrim_pressed: bool,
    hovered_destination: Option<usize>,
    pressed_destination: Option<usize>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for NavigationDrawer<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DrawerState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DrawerState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.host)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.host));
    }

    fn size(&self) -> Size<Length> {
        self.host.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.host
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
        self.host.as_widget().draw(
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
        if !(self.expanded && self.modal) {
            self.host.as_widget_mut().update(
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
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.expanded && self.modal {
            mouse::Interaction::default()
        } else {
            self.host.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.host
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if !(self.expanded && self.modal) {
            return self.host.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            );
        }

        Some(overlay::Element::new(Box::new(DrawerOverlay {
            items: &self.items,
            active: self.active,
            on_dismiss: self.on_dismiss.as_ref(),
            on_select: self.on_select.as_deref(),
            state: tree.state.downcast_mut(),
            style_fn: &self.class,
            viewport: *viewport,
            _renderer: std::marker::PhantomData,
        })))
    }
}

/// The overlay rendered when the drawer is open in modal mode.
struct DrawerOverlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    items: &'b [DrawerItem],
    active: Option<usize>,
    on_dismiss: Option<&'b Message>,
    on_select: Option<&'b (dyn Fn(usize) -> Message + 'a)>,
    state: &'b mut DrawerState,
    style_fn: &'b <Theme as Catalog>::Class<'a>,
    viewport: Rectangle,
    _renderer: std::marker::PhantomData<Renderer>,
}

/// Returns the destination index for a given item index, or `None` if
/// the item is not a destination.
fn destination_index_at(items: &[DrawerItem], item_idx: usize) -> Option<usize> {
    let mut dest_idx = 0;
    for (i, item) in items.iter().enumerate() {
        if i == item_idx {
            return match item {
                DrawerItem::Destination(_) => Some(dest_idx),
                _ => None,
            };
        }
        if matches!(item, DrawerItem::Destination(_)) {
            dest_idx += 1;
        }
    }
    None
}

/// Returns the y-offset and item index for each item.
fn item_rects(items: &[DrawerItem], drawer_y: f32) -> Vec<(usize, f32, f32)> {
    let mut y = drawer_y + DRAWER_PADDING;
    let mut rects = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let h = match item {
            DrawerItem::Destination(_) => DESTINATION_HEIGHT,
            DrawerItem::Header(_) => HEADER_HEIGHT,
            DrawerItem::Divider => DIVIDER_HEIGHT,
        };
        rects.push((i, y, h));
        y += h;
    }
    rects
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for DrawerOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog + SpacingBase + RoundnessBase,
    Renderer: renderer::Renderer + text::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        let viewport_size = self.viewport.size();

        // Drawer panel child node
        let drawer_height = viewport_size.height;
        let drawer_width = DRAWER_WIDTH.min(viewport_size.width - 56.0);
        let drawer_node =
            layout::Node::new(Size::new(drawer_width, drawer_height)).move_to(Point::ORIGIN);

        layout::Node::with_children(viewport_size, vec![drawer_node])
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let drawer_style = <Theme as Catalog>::style(theme, self.style_fn);
        let bounds = layout.bounds();

        // Scrim
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..renderer::Quad::default()
            },
            iced::Background::Color(drawer_style.scrim_color),
        );

        // Drawer panel
        let drawer_layout = layout.children().next().unwrap();
        let drawer_bounds = drawer_layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds: drawer_bounds,
                border: drawer_style.border,
                shadow: drawer_style.shadow,
                ..renderer::Quad::default()
            },
            drawer_style.background,
        );

        // Draw items
        let rects = item_rects(self.items, drawer_bounds.y);
        let mut dest_idx: usize = 0;

        for (i, y, h) in &rects {
            let item = &self.items[*i];
            match item {
                DrawerItem::Destination(label) => {
                    let is_active = self.active == Some(dest_idx);

                    // Active indicator pill
                    if is_active {
                        let indicator_bounds = Rectangle {
                            x: drawer_bounds.x + DRAWER_PADDING,
                            y: *y + 4.0,
                            width: drawer_bounds.width - DRAWER_PADDING * 2.0,
                            height: h - 8.0,
                        };
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: indicator_bounds,
                                border: Border {
                                    radius: ((*h - 8.0) / 2.0).into(),
                                    ..Border::default()
                                },
                                ..renderer::Quad::default()
                            },
                            iced::Background::Color(drawer_style.active_indicator_color),
                        );
                    }

                    let text_color = if is_active {
                        drawer_style.active_text_color
                    } else {
                        drawer_style.inactive_text_color
                    };

                    renderer.fill_text(
                        Text {
                            content: label.clone(),
                            bounds: Size::new(drawer_bounds.width - DRAWER_PADDING * 4.0, *h),
                            size: Pixels(14.0),
                            line_height: text::LineHeight::Relative(1.0),
                            font: renderer.default_font(),
                            align_x: iced::alignment::Horizontal::Left.into(),
                            align_y: iced::alignment::Vertical::Center,
                            shaping: text::Shaping::Basic,
                            wrapping: text::Wrapping::None,
                        },
                        Point::new(drawer_bounds.x + DRAWER_PADDING * 2.0, *y + *h / 2.0),
                        text_color,
                        drawer_bounds,
                    );

                    dest_idx += 1;
                }
                DrawerItem::Divider => {
                    let line_y = *y + (*h / 2.0);
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: drawer_bounds.x + DRAWER_PADDING,
                                y: line_y,
                                width: drawer_bounds.width - DRAWER_PADDING * 2.0,
                                height: 1.0,
                            },
                            ..renderer::Quad::default()
                        },
                        iced::Background::Color(drawer_style.inactive_text_color),
                    );
                }
                DrawerItem::Header(header_text) => {
                    renderer.fill_text(
                        Text {
                            content: header_text.clone(),
                            bounds: Size::new(drawer_bounds.width - DRAWER_PADDING * 4.0, *h),
                            size: Pixels(14.0),
                            line_height: text::LineHeight::Relative(1.0),
                            font: renderer.default_font(),
                            align_x: iced::alignment::Horizontal::Left.into(),
                            align_y: iced::alignment::Vertical::Center,
                            shaping: text::Shaping::Basic,
                            wrapping: text::Wrapping::None,
                        },
                        Point::new(drawer_bounds.x + DRAWER_PADDING * 2.0, *y + *h / 2.0),
                        drawer_style.inactive_text_color,
                        drawer_bounds,
                    );
                }
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
        let bounds = layout.bounds();
        let drawer_layout = layout.children().next().unwrap();
        let drawer_bounds = drawer_layout.bounds();

        let is_over_drawer = cursor.is_over(drawer_bounds);
        let is_over_scrim = cursor.is_over(bounds) && !is_over_drawer;

        // Determine which destination the cursor is over
        let hovered_dest = if is_over_drawer {
            cursor.position().and_then(|pos| {
                let rects = item_rects(self.items, drawer_bounds.y);
                for (i, y, h) in &rects {
                    if pos.y >= *y && pos.y < *y + *h {
                        return destination_index_at(self.items, *i);
                    }
                }
                None
            })
        } else {
            None
        };

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.state.hovered_destination = hovered_dest;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(dest) = hovered_dest {
                    self.state.pressed_destination = Some(dest);
                } else if is_over_scrim {
                    self.state.scrim_pressed = true;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(pressed) = self.state.pressed_destination {
                    if self.state.hovered_destination == Some(pressed)
                        && let Some(on_select) = self.on_select
                    {
                        shell.publish(on_select(pressed));
                    }
                } else if self.state.scrim_pressed
                    && is_over_scrim
                    && let Some(msg) = self.on_dismiss
                {
                    shell.publish(msg.clone());
                }
                self.state.scrim_pressed = false;
                self.state.pressed_destination = None;
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
        let drawer_layout = layout.children().next().unwrap();
        let drawer_bounds = drawer_layout.bounds();

        if self.state.hovered_destination.is_some() {
            mouse::Interaction::Pointer
        } else if cursor.is_over(drawer_bounds) {
            mouse::Interaction::default()
        } else if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<NavigationDrawer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn from(drawer: NavigationDrawer<'a, Message, Theme, Renderer>) -> Self {
        Element::new(drawer)
    }
}
