//! Panel that slides from the bottom of its host content.
//!
//! The [`BottomSheet`] widget wraps "host" content. When `expanded` is
//! `true`, a sheet panel is rendered at the bottom via the overlay
//! system. Optionally a scrim covers the host when `modal` is `true`.
//!
//! The drag handle (enabled by default) allows the user to resize the
//! sheet by dragging vertically. Dragging below the minimum height
//! fraction (default 10%) dismisses the sheet.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::bottom_sheet::BottomSheet;
//!
//! let sheet = BottomSheet::new(host_content, "Sheet body text")
//!     .modal(true)
//!     .expanded(self.sheet_expanded)
//!     .on_dismiss(Message::CloseSheet)
//!     .on_resize(Message::SheetResized)
//!     .drag_handle(true);
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
use iced::{Color, Element, Event, Length, Pixels, Point, Rectangle, Size, Vector};

use crate::{FontSizeBase, RoundnessBase, SpacingBase};

/// Width of the drag handle pill in logical pixels.
const HANDLE_WIDTH: f32 = 32.0;
/// Height of the drag handle pill in logical pixels.
const HANDLE_HEIGHT: f32 = 4.0;
/// Vertical padding above/below the drag handle.
const HANDLE_VERTICAL_PADDING: f32 = 12.0;
/// Inner padding for sheet content.
const SHEET_PADDING: f32 = 24.0;
/// Height of the hit zone for grabbing the drag handle.
const HANDLE_HIT_HEIGHT: f32 = HANDLE_HEIGHT + HANDLE_VERTICAL_PADDING * 2.0;
/// Default minimum height fraction; below this the sheet will dismiss.
const DEFAULT_MIN_HEIGHT_FRACTION: f32 = 0.1;

/// Wraps host content and conditionally displays a sheet panel from
/// the bottom of its bounds.
pub struct BottomSheet<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    host: Element<'a, Message, Theme, Renderer>,
    sheet_body: String,
    modal: bool,
    expanded: bool,
    on_dismiss: Option<Message>,
    on_resize: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    drag_handle: bool,
    height_fraction: f32,
    min_height_fraction: f32,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> BottomSheet<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new bottom sheet wrapping the given host content with
    /// the provided sheet body text.
    pub fn new(
        host: impl Into<Element<'a, Message, Theme, Renderer>>,
        sheet_body: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            sheet_body: sheet_body.into(),
            modal: false,
            expanded: false,
            on_dismiss: None,
            on_resize: None,
            drag_handle: true,
            height_fraction: 0.5,
            min_height_fraction: DEFAULT_MIN_HEIGHT_FRACTION,
            class: Theme::default(),
        }
    }

    /// Controls whether the sheet is modal (shows scrim over host).
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Controls whether the sheet is expanded (visible).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Sets the message emitted when the scrim is pressed to dismiss,
    /// or when the sheet is dragged below the minimum height.
    pub fn on_dismiss(mut self, msg: Message) -> Self {
        self.on_dismiss = Some(msg);
        self
    }

    /// Sets a callback invoked with the new height fraction when the
    /// user finishes resizing the sheet via the drag handle. Optional;
    /// drag-to-resize works regardless of whether this is set.
    pub fn on_resize(mut self, f: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_resize = Some(Box::new(f));
        self
    }

    /// Controls whether the drag handle pill is shown.
    pub fn drag_handle(mut self, show: bool) -> Self {
        self.drag_handle = show;
        self
    }

    /// Sets the sheet height as a fraction of the host (0.0–1.0).
    /// Defaults to 0.5 (50%).
    pub fn height_fraction(mut self, fraction: f32) -> Self {
        self.height_fraction = fraction.clamp(0.1, 1.0);
        self
    }

    /// Sets the minimum height fraction before the sheet dismisses.
    /// Defaults to 0.1 (10%). Must be in range (0.0, 1.0).
    pub fn min_height_fraction(mut self, fraction: f32) -> Self {
        self.min_height_fraction = fraction.clamp(0.01, 0.99);
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// State for the bottom sheet overlay interaction.
#[derive(Debug, Default)]
struct SheetState {
    scrim_pressed: bool,
    /// Whether the user is currently dragging the handle.
    dragging: bool,
    /// The absolute Y coordinate where the drag started.
    drag_start_y: f32,
    /// The height fraction at the moment the drag started.
    drag_start_fraction: f32,
    /// Live height fraction override while dragging (or after drag).
    height_override: Option<f32>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for BottomSheet<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SheetState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SheetState::default())
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
        if !self.expanded || !self.modal {
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
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if !self.expanded {
            return self.host.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                _viewport,
                translation,
            );
        }

        // Compute the host's absolute on-screen bounds, accounting for
        // any scroll offset via the translation vector.
        let mut host_bounds = layout.bounds();
        host_bounds.x += translation.x;
        host_bounds.y += translation.y;

        Some(overlay::Element::new(Box::new(SheetOverlay {
            sheet_body: &self.sheet_body,
            modal: self.modal,
            on_dismiss: self.on_dismiss.as_ref(),
            on_resize: self.on_resize.as_deref(),
            drag_handle: self.drag_handle,
            height_fraction: self.height_fraction,
            min_height_fraction: self.min_height_fraction,
            state: tree.state.downcast_mut(),
            style_fn: &self.class,
            host_bounds,
            _renderer: std::marker::PhantomData,
        })))
    }
}

/// The overlay rendered when the bottom sheet is expanded.
struct SheetOverlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    sheet_body: &'b str,
    modal: bool,
    on_dismiss: Option<&'b Message>,
    on_resize: Option<&'b (dyn Fn(f32) -> Message + 'a)>,
    drag_handle: bool,
    height_fraction: f32,
    min_height_fraction: f32,
    state: &'b mut SheetState,
    style_fn: &'b <Theme as Catalog>::Class<'a>,
    host_bounds: Rectangle,
    _renderer: std::marker::PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer> SheetOverlay<'a, '_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Returns the effective height fraction, preferring the live drag
    /// override when active.
    fn effective_fraction(&self) -> f32 {
        self.state
            .height_override
            .unwrap_or(self.height_fraction)
            .clamp(0.0, 1.0)
    }

    /// Computes the handle hit zone rectangle given the sheet bounds.
    fn handle_hit_rect(sheet_bounds: &Rectangle) -> Rectangle {
        Rectangle {
            x: sheet_bounds.x,
            y: sheet_bounds.y,
            width: sheet_bounds.width,
            height: HANDLE_HIT_HEIGHT,
        }
    }
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for SheetOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase,
    Renderer: renderer::Renderer + text::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        let host = self.host_bounds;
        let fraction = self.effective_fraction();

        let sheet_height = (host.height * fraction).max(0.0).min(host.height);
        let sheet_y = host.height - sheet_height;

        // Sheet child node — position is relative to root node.
        let sheet_node = layout::Node::new(Size::new(host.width, sheet_height))
            .move_to(Point::new(0.0, sheet_y));

        // Root node covers the host area at its absolute position.
        layout::Node::with_children(Size::new(host.width, host.height), vec![sheet_node])
            .move_to(Point::new(host.x, host.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let sheet_style = <Theme as Catalog>::style(theme, self.style_fn);
        let bounds = layout.bounds();
        let fraction = self.effective_fraction();

        // Scrim (if modal)
        if self.modal {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..renderer::Quad::default()
                },
                iced::Background::Color(sheet_style.scrim_color),
            );
        }

        // Sheet panel
        let sheet_layout = layout.children().next().unwrap();
        let sheet_bounds = sheet_layout.bounds();

        // Compute dismiss fade: when fraction is below minimum, fade
        // out the sheet to indicate impending dismissal.
        let dismiss_alpha = if fraction < self.min_height_fraction {
            // Linear fade from 1.0 (at min) to 0.0 (at 0)
            (fraction / self.min_height_fraction).clamp(0.0, 1.0)
        } else {
            1.0
        };

        // Apply dismiss alpha to the background color
        let bg_color = match sheet_style.background {
            iced::Background::Color(c) => Color {
                a: c.a * dismiss_alpha,
                ..c
            },
            other => match other {
                iced::Background::Color(c) => Color {
                    a: c.a * dismiss_alpha,
                    ..c
                },
                // Gradient backgrounds cannot have alpha easily modified;
                // fall back to the original.
                _ => Color::TRANSPARENT,
            },
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: sheet_bounds,
                border: sheet_style.border,
                shadow: sheet_style.shadow,
                ..renderer::Quad::default()
            },
            iced::Background::Color(bg_color),
        );

        let mut y_cursor = sheet_bounds.y;

        // Drag handle
        if self.drag_handle {
            y_cursor += HANDLE_VERTICAL_PADDING;
            let handle_x = sheet_bounds.x + (sheet_bounds.width - HANDLE_WIDTH) / 2.0;

            let handle_color = Color {
                a: sheet_style.handle_color.a * dismiss_alpha,
                ..sheet_style.handle_color
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: handle_x,
                        y: y_cursor,
                        width: HANDLE_WIDTH,
                        height: HANDLE_HEIGHT,
                    },
                    border: iced::Border {
                        radius: (HANDLE_HEIGHT / 2.0).into(),
                        ..iced::Border::default()
                    },
                    ..renderer::Quad::default()
                },
                iced::Background::Color(handle_color),
            );

            y_cursor += HANDLE_HEIGHT + HANDLE_VERTICAL_PADDING;
        } else {
            y_cursor += SHEET_PADDING;
        }

        // Body text
        let text_x = sheet_bounds.x + SHEET_PADDING;
        let text_width = sheet_bounds.width - SHEET_PADDING * 2.0;
        let text_height = sheet_bounds.height - (y_cursor - sheet_bounds.y) - SHEET_PADDING;

        let text_color = Color {
            a: sheet_style.handle_color.a * dismiss_alpha,
            ..sheet_style.handle_color
        };

        renderer.fill_text(
            Text {
                content: self.sheet_body.to_string(),
                bounds: Size::new(text_width.max(0.0), text_height.max(0.0)),
                size: Pixels(theme.text_size() * 0.875),
                line_height: text::LineHeight::Relative(1.4),
                font: renderer.default_font(),
                align_x: iced::alignment::Horizontal::Left.into(),
                align_y: iced::alignment::Vertical::Top,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::WordOrGlyph,
            },
            Point::new(text_x, y_cursor),
            text_color,
            sheet_bounds,
        );
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
        let sheet_layout = layout.children().next().unwrap();
        let sheet_bounds = sheet_layout.bounds();

        let is_over_sheet = cursor.is_over(sheet_bounds);
        let is_over_scrim = cursor.is_over(bounds) && !is_over_sheet;

        let handle_hit = if self.drag_handle {
            let hit_rect = Self::handle_hit_rect(&sheet_bounds);
            cursor.is_over(hit_rect)
        } else {
            false
        };

        match event {
            // --- Drag handle interaction ---
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if handle_hit && self.drag_handle =>
            {
                if let Some(pos) = cursor.position() {
                    self.state.dragging = true;
                    self.state.drag_start_y = pos.y;
                    self.state.drag_start_fraction = self.effective_fraction();
                    shell.capture_event();
                }
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) if self.state.dragging => {
                if let Some(pos) = cursor.position() {
                    let host_height = self.host_bounds.height;
                    if host_height > 0.0 {
                        let delta_y = pos.y - self.state.drag_start_y;
                        // Dragging down shrinks the sheet
                        let new_fraction = self.state.drag_start_fraction - (delta_y / host_height);
                        self.state.height_override = Some(new_fraction.clamp(0.0, 1.0));
                        shell.request_redraw();
                    }
                    shell.capture_event();
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if self.state.dragging =>
            {
                self.state.dragging = false;
                let final_fraction = self.effective_fraction();

                if final_fraction < self.min_height_fraction {
                    // Dismiss the sheet
                    self.state.height_override = None;
                    if let Some(msg) = self.on_dismiss {
                        shell.publish(msg.clone());
                    }
                } else {
                    // Publish the final fraction via on_resize
                    if let Some(on_resize) = self.on_resize {
                        shell.publish(on_resize(final_fraction));
                    }
                    self.state.height_override = None;
                }

                shell.capture_event();
            }

            // --- Scrim dismiss interaction ---
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if is_over_scrim && self.modal && !self.state.dragging =>
            {
                self.state.scrim_pressed = true;
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if !self.state.dragging =>
            {
                if self.state.scrim_pressed
                    && is_over_scrim
                    && let Some(msg) = self.on_dismiss
                {
                    shell.publish(msg.clone());
                }
                self.state.scrim_pressed = false;
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
        if self.state.dragging {
            return mouse::Interaction::ResizingVertically;
        }

        let sheet_layout = layout.children().next().unwrap();
        let sheet_bounds = sheet_layout.bounds();

        if self.drag_handle {
            let hit_rect = Self::handle_hit_rect(&sheet_bounds);
            if cursor.is_over(hit_rect) {
                return mouse::Interaction::ResizingVertically;
            }
        }

        if cursor.is_over(sheet_bounds) {
            mouse::Interaction::default()
        } else if self.modal && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<BottomSheet<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn from(sheet: BottomSheet<'a, Message, Theme, Renderer>) -> Self {
        Element::new(sheet)
    }
}
