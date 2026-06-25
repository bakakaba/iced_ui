//! A panel that slides from any edge of its host content.
//!
//! The [`SlideSheet`] widget wraps "host" content. When `expanded` is
//! `true`, a sheet panel slides in from the configured [`Anchor`] edge
//! via the overlay system.
//!
//! The drag handle (enabled by default) allows the user to resize the
//! sheet by dragging along the sheet's axis. Dragging below the
//! minimum size fraction (default 10%) dismisses the sheet.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::slide_sheet::{SlideSheet, Anchor};
//!
//! let sheet = SlideSheet::new(host_content, "Sheet body text")
//!     .anchor(Anchor::Bottom)
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

use crate::{FontSizeBase, RoundnessBase, Space, SpacingBase};

/// Length of the drag handle pill along its major axis (logical pixels).
const HANDLE_LENGTH: f32 = 32.0;
/// Thickness of the drag handle pill (logical pixels).
const HANDLE_THICKNESS: f32 = 4.0;
/// Padding between the handle pill and the sheet edge / content.
const HANDLE_PADDING: f32 = 12.0;
/// Depth of the hit zone for grabbing the drag handle (along the
/// sheet's slide axis).
const HANDLE_HIT_DEPTH: f32 = HANDLE_THICKNESS + HANDLE_PADDING * 2.0;
/// Default minimum size fraction; below this the sheet will dismiss.
const DEFAULT_MIN_SIZE: f32 = 0.1;

/// The edge from which the sheet slides in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    /// Sheet slides from the bottom edge.
    #[default]
    Bottom,
    /// Sheet slides from the top edge.
    Top,
    /// Sheet slides from the left edge.
    Left,
    /// Sheet slides from the right edge.
    Right,
}

impl Anchor {
    /// Whether this anchor slides along the vertical axis.
    fn is_vertical(self) -> bool {
        matches!(self, Self::Bottom | Self::Top)
    }

    /// Returns the appropriate resize cursor for this anchor.
    fn resize_cursor(self) -> mouse::Interaction {
        if self.is_vertical() {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::ResizingHorizontally
        }
    }

    /// Returns the shadow offset vector pointing away from the anchor,
    /// at the given magnitude (the theme-resolved shadow distance).
    fn shadow_offset(self, distance: f32) -> Vector {
        match self {
            Self::Bottom => Vector::new(0.0, -distance),
            Self::Top => Vector::new(0.0, distance),
            Self::Left => Vector::new(distance, 0.0),
            Self::Right => Vector::new(-distance, 0.0),
        }
    }

    /// Zeros the border radius on corners that sit flush against the
    /// anchor edge.
    fn apply_border_radius(self, radius: &mut iced::border::Radius) {
        match self {
            Self::Bottom => {
                radius.bottom_left = 0.0;
                radius.bottom_right = 0.0;
            }
            Self::Top => {
                radius.top_left = 0.0;
                radius.top_right = 0.0;
            }
            Self::Left => {
                radius.top_left = 0.0;
                radius.bottom_left = 0.0;
            }
            Self::Right => {
                radius.top_right = 0.0;
                radius.bottom_right = 0.0;
            }
        }
    }
}

/// Wraps host content and conditionally displays a sheet panel from
/// a configured edge of its bounds.
pub struct SlideSheet<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    host: Element<'a, Message, Theme, Renderer>,
    sheet_body: String,
    anchor: Anchor,
    expanded: bool,
    on_dismiss: Option<Message>,
    on_resize: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    drag_handle: bool,
    size: f32,
    min_size: f32,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> SlideSheet<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new slide sheet wrapping the given host content with
    /// the provided sheet body text.
    pub fn new(
        host: impl Into<Element<'a, Message, Theme, Renderer>>,
        sheet_body: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            sheet_body: sheet_body.into(),
            anchor: Anchor::default(),
            expanded: false,
            on_dismiss: None,
            on_resize: None,
            drag_handle: true,
            size: 0.5,
            min_size: DEFAULT_MIN_SIZE,
            class: Theme::default(),
        }
    }

    /// Sets the edge from which the sheet slides in.
    /// Defaults to [`Anchor::Bottom`].
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Controls whether the sheet is expanded (visible).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Sets the message emitted when the sheet is dragged below the
    /// minimum size and dismissed.
    pub fn on_dismiss(mut self, msg: Message) -> Self {
        self.on_dismiss = Some(msg);
        self
    }

    /// Sets a callback invoked with the new size fraction when the
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

    /// Sets the sheet size as a fraction of the screen along the
    /// sheet's axis (0.0–1.0). Defaults to 0.5 (50%).
    ///
    /// For [`Anchor::Bottom`] and [`Anchor::Top`] this controls
    /// the sheet's height. For [`Anchor::Left`] and [`Anchor::Right`]
    /// this controls the sheet's width.
    pub fn size(mut self, fraction: f32) -> Self {
        self.size = fraction.clamp(0.1, 1.0);
        self
    }

    /// Sets the minimum size fraction before the sheet dismisses.
    /// Defaults to 0.1 (10%). Must be in range (0.0, 1.0).
    pub fn min_size(mut self, fraction: f32) -> Self {
        self.min_size = fraction.clamp(0.01, 0.99);
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// State for the slide sheet overlay interaction.
#[derive(Debug, Default)]
struct SheetState {
    /// Whether the user is currently dragging the handle.
    dragging: bool,
    /// The position (on the sheet's axis) where the drag started.
    drag_start_pos: f32,
    /// The size fraction at the moment the drag started.
    drag_start_size: f32,
    /// Live size fraction override while dragging.
    size_override: Option<f32>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SlideSheet<'a, Message, Theme, Renderer>
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
        // Host always receives events — no modal blocking.
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

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.host.as_widget().mouse_interaction(
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
            anchor: self.anchor,
            on_dismiss: self.on_dismiss.as_ref(),
            on_resize: self.on_resize.as_deref(),
            drag_handle: self.drag_handle,
            size: self.size,
            min_size: self.min_size,
            state: tree.state.downcast_mut(),
            style_fn: &self.class,
            host_bounds,
            _renderer: std::marker::PhantomData,
        })))
    }
}

/// The overlay rendered when the slide sheet is expanded.
struct SheetOverlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    sheet_body: &'b str,
    anchor: Anchor,
    on_dismiss: Option<&'b Message>,
    on_resize: Option<&'b (dyn Fn(f32) -> Message + 'a)>,
    drag_handle: bool,
    size: f32,
    min_size: f32,
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
    /// Returns the effective size fraction, preferring the live drag
    /// override when active.
    fn effective_size(&self) -> f32 {
        self.state
            .size_override
            .unwrap_or(self.size)
            .clamp(0.0, 1.0)
    }

    /// Computes the handle hit zone rectangle given the sheet bounds.
    /// The hit zone is at the edge closest to the host content (the
    /// sliding edge).
    fn handle_hit_rect(&self, sheet_bounds: &Rectangle) -> Rectangle {
        match self.anchor {
            Anchor::Bottom => Rectangle {
                x: sheet_bounds.x,
                y: sheet_bounds.y,
                width: sheet_bounds.width,
                height: HANDLE_HIT_DEPTH,
            },
            Anchor::Top => Rectangle {
                x: sheet_bounds.x,
                y: sheet_bounds.y + sheet_bounds.height - HANDLE_HIT_DEPTH,
                width: sheet_bounds.width,
                height: HANDLE_HIT_DEPTH,
            },
            Anchor::Left => Rectangle {
                x: sheet_bounds.x + sheet_bounds.width - HANDLE_HIT_DEPTH,
                y: sheet_bounds.y,
                width: HANDLE_HIT_DEPTH,
                height: sheet_bounds.height,
            },
            Anchor::Right => Rectangle {
                x: sheet_bounds.x,
                y: sheet_bounds.y,
                width: HANDLE_HIT_DEPTH,
                height: sheet_bounds.height,
            },
        }
    }

    /// Extracts the relevant coordinate from a position based on
    /// the anchor's axis.
    fn axis_pos(&self, pos: Point) -> f32 {
        if self.anchor.is_vertical() {
            pos.y
        } else {
            pos.x
        }
    }

    /// Returns the host dimension along the sheet's axis.
    fn host_axis_size(&self) -> f32 {
        if self.anchor.is_vertical() {
            self.host_bounds.height
        } else {
            self.host_bounds.width
        }
    }

    /// Computes the new size fraction from a drag delta.
    fn compute_drag_fraction(&self, delta: f32) -> f32 {
        let axis_size = self.host_axis_size();
        if axis_size <= 0.0 {
            return self.state.drag_start_size;
        }

        let fraction_delta = delta / axis_size;

        // Determine sign: for Bottom/Right, moving toward the anchor
        // edge (positive delta) shrinks the sheet. For Top/Left,
        // moving toward the anchor edge (negative delta) shrinks.
        let new = match self.anchor {
            Anchor::Bottom | Anchor::Right => self.state.drag_start_size - fraction_delta,
            Anchor::Top | Anchor::Left => self.state.drag_start_size + fraction_delta,
        };

        new.clamp(0.0, 1.0)
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
        let fraction = self.effective_size();

        let (sheet_size, sheet_pos) = match self.anchor {
            Anchor::Bottom => {
                let h = (host.height * fraction).clamp(0.0, host.height);
                (Size::new(host.width, h), Point::new(0.0, host.height - h))
            }
            Anchor::Top => {
                let h = (host.height * fraction).clamp(0.0, host.height);
                (Size::new(host.width, h), Point::ORIGIN)
            }
            Anchor::Left => {
                let w = (host.width * fraction).clamp(0.0, host.width);
                (Size::new(w, host.height), Point::ORIGIN)
            }
            Anchor::Right => {
                let w = (host.width * fraction).clamp(0.0, host.width);
                (Size::new(w, host.height), Point::new(host.width - w, 0.0))
            }
        };

        let sheet_node = layout::Node::new(sheet_size).move_to(sheet_pos);

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
        let mut sheet_style = <Theme as Catalog>::style(theme, self.style_fn);
        let fraction = self.effective_size();

        // Apply anchor-specific border radius and shadow direction.
        // The style produced a downward shadow; re-orient its offset to
        // point away from the sheet's anchor, preserving the
        // theme-resolved magnitude.
        self.anchor
            .apply_border_radius(&mut sheet_style.border.radius);
        let distance = sheet_style.shadow.offset.y;
        sheet_style.shadow.offset = self.anchor.shadow_offset(distance);

        // Sheet panel
        let sheet_layout = layout.children().next().unwrap();
        let sheet_bounds = sheet_layout.bounds();

        // Compute dismiss fade: when fraction is below minimum,
        // apply a fixed low alpha to the entire sheet to indicate
        // impending dismissal.
        let dismiss_alpha = if fraction < self.min_size { 0.3 } else { 1.0 };

        // Apply dismiss alpha to the background and shadow.
        let bg_color = match sheet_style.background {
            iced::Background::Color(c) => Color {
                a: c.a * dismiss_alpha,
                ..c
            },
            _ => Color::TRANSPARENT,
        };

        sheet_style.shadow.color.a *= dismiss_alpha;

        renderer.fill_quad(
            renderer::Quad {
                bounds: sheet_bounds,
                border: sheet_style.border,
                shadow: sheet_style.shadow,
                ..renderer::Quad::default()
            },
            iced::Background::Color(bg_color),
        );

        // Drag handle and content positioning depend on anchor.
        let handle_color = Color {
            a: sheet_style.handle_color.a * dismiss_alpha,
            ..sheet_style.handle_color
        };

        let text_color = Color {
            a: sheet_style.handle_color.a * dismiss_alpha,
            ..sheet_style.handle_color
        };

        // Resolve content padding from theme spacing.
        let padding = Space::sx(2.0).resolve(theme.spacing());

        // For vertical anchors (Bottom/Top): handle is horizontal,
        // content flows vertically.
        // For horizontal anchors (Left/Right): handle is vertical,
        // content flows horizontally.
        let content_rect = if self.anchor.is_vertical() {
            self.draw_vertical(renderer, &sheet_bounds, handle_color, padding)
        } else {
            self.draw_horizontal(renderer, &sheet_bounds, handle_color, padding)
        };

        // Body text
        if content_rect.width > 0.0 && content_rect.height > 0.0 {
            renderer.fill_text(
                Text {
                    content: self.sheet_body.to_string(),
                    bounds: Size::new(content_rect.width, content_rect.height),
                    size: Pixels(theme.text_size() * 0.875),
                    line_height: text::LineHeight::Relative(1.4),
                    font: renderer.default_font(),
                    align_x: iced::alignment::Horizontal::Left.into(),
                    align_y: iced::alignment::Vertical::Top,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::WordOrGlyph,
                },
                Point::new(content_rect.x, content_rect.y),
                text_color,
                sheet_bounds,
            );
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
        let sheet_layout = layout.children().next().unwrap();
        let sheet_bounds = sheet_layout.bounds();

        let handle_hit = if self.drag_handle {
            let hit_rect = self.handle_hit_rect(&sheet_bounds);
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
                    self.state.drag_start_pos = self.axis_pos(pos);
                    self.state.drag_start_size = self.effective_size();
                    shell.capture_event();
                }
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) if self.state.dragging => {
                if let Some(pos) = cursor.position() {
                    let current_pos = self.axis_pos(pos);
                    let delta = current_pos - self.state.drag_start_pos;
                    let new_fraction = self.compute_drag_fraction(delta);
                    self.state.size_override = Some(new_fraction);
                    shell.request_redraw();
                    shell.capture_event();
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if self.state.dragging =>
            {
                self.state.dragging = false;
                let final_size = self.effective_size();

                if final_size < self.min_size {
                    // Dismiss the sheet
                    self.state.size_override = None;
                    if let Some(msg) = self.on_dismiss {
                        shell.publish(msg.clone());
                    }
                } else {
                    // Publish the final fraction via on_resize
                    if let Some(on_resize) = self.on_resize {
                        shell.publish(on_resize(final_size));
                    }
                    self.state.size_override = None;
                }

                shell.capture_event();
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
            return self.anchor.resize_cursor();
        }

        let sheet_layout = layout.children().next().unwrap();
        let sheet_bounds = sheet_layout.bounds();

        if self.drag_handle {
            let hit_rect = self.handle_hit_rect(&sheet_bounds);
            if cursor.is_over(hit_rect) {
                return self.anchor.resize_cursor();
            }
        }

        mouse::Interaction::default()
    }
}

// --- Drawing helpers ---

impl<'a, Message, Theme, Renderer> SheetOverlay<'a, '_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + text::Renderer,
{
    /// Draws handle for vertical anchors (Bottom/Top). Returns the
    /// content rectangle available for body text.
    fn draw_vertical(
        &self,
        renderer: &mut Renderer,
        sheet_bounds: &Rectangle,
        handle_color: Color,
        padding: f32,
    ) -> Rectangle {
        let mut content_y = sheet_bounds.y;
        let mut content_height = sheet_bounds.height;

        if self.drag_handle {
            // Handle position depends on anchor:
            // Bottom → handle at top of sheet
            // Top → handle at bottom of sheet
            let handle_y = match self.anchor {
                Anchor::Bottom => {
                    content_y += HANDLE_HIT_DEPTH;
                    content_height -= HANDLE_HIT_DEPTH;
                    sheet_bounds.y + HANDLE_PADDING
                }
                Anchor::Top => {
                    content_height -= HANDLE_HIT_DEPTH;
                    sheet_bounds.y + sheet_bounds.height - HANDLE_PADDING - HANDLE_THICKNESS
                }
                _ => unreachable!(),
            };

            let handle_x = sheet_bounds.x + (sheet_bounds.width - HANDLE_LENGTH) / 2.0;

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: handle_x,
                        y: handle_y,
                        width: HANDLE_LENGTH,
                        height: HANDLE_THICKNESS,
                    },
                    border: iced::Border {
                        radius: (HANDLE_THICKNESS / 2.0).into(),
                        ..iced::Border::default()
                    },
                    ..renderer::Quad::default()
                },
                iced::Background::Color(handle_color),
            );
        } else {
            // No handle — account for padding at the anchor edge
            match self.anchor {
                Anchor::Bottom => {
                    content_y += padding;
                    content_height -= padding;
                }
                Anchor::Top => {
                    content_height -= padding;
                }
                _ => unreachable!(),
            }
        }

        // Uniform padding on all remaining sides
        Rectangle {
            x: sheet_bounds.x + padding,
            y: content_y + padding,
            width: (sheet_bounds.width - padding * 2.0).max(0.0),
            height: (content_height - padding * 2.0).max(0.0),
        }
    }

    /// Draws handle for horizontal anchors (Left/Right). Returns the
    /// content rectangle available for body text.
    fn draw_horizontal(
        &self,
        renderer: &mut Renderer,
        sheet_bounds: &Rectangle,
        handle_color: Color,
        padding: f32,
    ) -> Rectangle {
        let mut content_x = sheet_bounds.x;
        let mut content_width = sheet_bounds.width;

        if self.drag_handle {
            // Handle position depends on anchor:
            // Left → handle at right edge of sheet
            // Right → handle at left edge of sheet
            let handle_x = match self.anchor {
                Anchor::Left => {
                    content_width -= HANDLE_HIT_DEPTH;
                    sheet_bounds.x + sheet_bounds.width - HANDLE_PADDING - HANDLE_THICKNESS
                }
                Anchor::Right => {
                    content_x += HANDLE_HIT_DEPTH;
                    content_width -= HANDLE_HIT_DEPTH;
                    sheet_bounds.x + HANDLE_PADDING
                }
                _ => unreachable!(),
            };

            let handle_y = sheet_bounds.y + (sheet_bounds.height - HANDLE_LENGTH) / 2.0;

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: handle_x,
                        y: handle_y,
                        width: HANDLE_THICKNESS,
                        height: HANDLE_LENGTH,
                    },
                    border: iced::Border {
                        radius: (HANDLE_THICKNESS / 2.0).into(),
                        ..iced::Border::default()
                    },
                    ..renderer::Quad::default()
                },
                iced::Background::Color(handle_color),
            );
        } else {
            // No handle — account for padding at the anchor edge
            match self.anchor {
                Anchor::Left => {
                    content_width -= padding;
                }
                Anchor::Right => {
                    content_x += padding;
                    content_width -= padding;
                }
                _ => unreachable!(),
            }
        }

        // Uniform padding on all remaining sides
        Rectangle {
            x: content_x + padding,
            y: sheet_bounds.y + padding,
            width: (content_width - padding * 2.0).max(0.0),
            height: (sheet_bounds.height - padding * 2.0).max(0.0),
        }
    }
}

impl<'a, Message, Theme, Renderer> From<SlideSheet<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn from(sheet: SlideSheet<'a, Message, Theme, Renderer>) -> Self {
        Element::new(sheet)
    }
}
