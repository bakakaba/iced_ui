//! Temporary notification bar overlaid on host content.
//!
//! The [`Snackbar`] widget wraps "host" content. When `visible` is
//! `true`, a floating bar is rendered via the overlay system, showing
//! a message and optional action button.
//!
//! # Features
//!
//! - **Anchor** — position the bar at any corner or edge via
//!   [`Position`].
//! - **Severity** — color-coded left border + icon to indicate
//!   information, success, warning, or error states.
//! - **Auto-dismiss** — countdown timer that automatically hides the
//!   snackbar after a configured duration.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::snackbar::{Snackbar, Severity};
//! use iced_ui::Position;
//! use std::time::Duration;
//!
//! let snackbar = Snackbar::new(host_content)
//!     .message("Item deleted")
//!     .action("Undo", Message::Undo)
//!     .on_dismiss(Message::DismissSnackbar)
//!     .anchor(Position::BottomRight)
//!     .severity(Severity::Success)
//!     .auto_dismiss(Duration::from_secs(5))
//!     .visible(self.snackbar_visible);
//! ```

mod style;

pub use style::{Catalog, Style, StyleFn, default};

use std::time::{Duration, Instant};

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::text::{self, Text};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::window;
use iced::{Color, Element, Event, Length, Pixels, Point, Rectangle, Size, Vector};

use crate::position::Position;
use crate::{FontSizeBase, RoundnessBase, SpacingBase};

/// Height of the snackbar bar in logical pixels.
const BAR_HEIGHT: f32 = 48.0;
/// Margin from the edges of the viewport.
const EDGE_MARGIN: f32 = 16.0;
/// Internal horizontal padding within the bar.
const INNER_PADDING: f32 = 16.0;
/// Width of the severity accent border on the left.
const SEVERITY_BORDER_WIDTH: f32 = 4.0;
/// Width reserved for the severity icon area.
const ICON_AREA_WIDTH: f32 = 28.0;
/// Maximum bar width (prevents overly wide bars at corners).
const MAX_BAR_WIDTH: f32 = 560.0;

/// The severity level of a snackbar notification.
///
/// Controls the accent color (left border) and the icon displayed at
/// the start of the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Severity {
    /// No severity indicator (default). Dark background, no accent.
    #[default]
    Neutral,
    /// Informational — uses the information color token (cyan/blue).
    Information,
    /// Success — uses the success color token (green).
    Success,
    /// Warning — uses the warning color token (amber/yellow).
    Warning,
    /// Error — uses the danger color token (red).
    Error,
}

impl Severity {
    /// Returns a Unicode fallback glyph for this severity level.
    fn fallback_glyph(self) -> &'static str {
        match self {
            Self::Neutral => "",
            Self::Information => "\u{2139}", // ℹ
            Self::Success => "\u{2713}",     // ✓
            Self::Warning => "\u{26A0}",     // ⚠
            Self::Error => "\u{2715}",       // ✕
        }
    }
}

/// Wraps host content and conditionally displays a floating
/// notification bar with a message and optional action.
pub struct Snackbar<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    host: Element<'a, Message, Theme, Renderer>,
    message_text: Option<String>,
    action: Option<(String, Message)>,
    on_dismiss: Option<Message>,
    visible: bool,
    anchor: Position,
    severity: Severity,
    auto_dismiss: Option<Duration>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Snackbar<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new snackbar wrapping the given host content.
    pub fn new(host: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            host: host.into(),
            message_text: None,
            action: None,
            on_dismiss: None,
            visible: false,
            anchor: Position::BottomRight,
            severity: Severity::Neutral,
            auto_dismiss: None,
            class: Theme::default(),
        }
    }

    /// Sets the snackbar message text.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message_text = Some(message.into());
        self
    }

    /// Adds an action button with a label and message.
    pub fn action(mut self, label: impl Into<String>, msg: Message) -> Self {
        self.action = Some((label.into(), msg));
        self
    }

    /// Sets the message emitted when the snackbar is dismissed.
    pub fn on_dismiss(mut self, msg: Message) -> Self {
        self.on_dismiss = Some(msg);
        self
    }

    /// Controls whether the snackbar is visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets the anchor position for the snackbar bar.
    /// Defaults to [`Position::BottomRight`].
    pub fn anchor(mut self, position: Position) -> Self {
        self.anchor = position;
        self
    }

    /// Sets the severity level, adding an accent border and icon.
    /// Defaults to [`Severity::Neutral`] (no accent).
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Enables auto-dismiss with the given duration.
    ///
    /// When set, the snackbar displays a countdown and automatically
    /// publishes the `on_dismiss` message when the timer expires.
    /// Hovering over the countdown area reveals a close icon for
    /// immediate dismissal.
    pub fn auto_dismiss(mut self, duration: Duration) -> Self {
        self.auto_dismiss = Some(duration);
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// State for the snackbar overlay interaction.
#[derive(Debug, Default)]
struct SnackbarState {
    action_hovered: bool,
    action_pressed: bool,
    dismiss_hovered: bool,
    dismiss_pressed: bool,
    /// When the auto-dismiss countdown started.
    countdown_start: Option<Instant>,
    /// Whether the snackbar was visible on the previous frame.
    was_visible: bool,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Snackbar<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SnackbarState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SnackbarState::default())
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

        // Manage countdown state transitions at the widget level.
        let state = tree.state.downcast_mut::<SnackbarState>();

        if self.visible && !state.was_visible {
            // Just became visible — start countdown if auto_dismiss is set.
            if self.auto_dismiss.is_some() {
                state.countdown_start = None; // Will be set on first RedrawRequested
            }
        } else if !self.visible && state.was_visible {
            // Just became hidden — reset.
            state.countdown_start = None;
        }
        state.was_visible = self.visible;

        // Request redraws when visible with auto_dismiss to keep the timer ticking.
        if self.visible
            && let Some(duration) = self.auto_dismiss
            && let Event::Window(window::Event::RedrawRequested(now)) = event
        {
            if state.countdown_start.is_none() {
                state.countdown_start = Some(*now);
            }

            let elapsed = now.duration_since(state.countdown_start.unwrap());

            if elapsed >= duration {
                if let Some(msg) = &self.on_dismiss {
                    shell.publish(msg.clone());
                }
            } else {
                shell.request_redraw();
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
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if !self.visible {
            return self.host.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            );
        }

        // Split borrows: `state` and `children` are separate fields of Tree.
        let Tree {
            state, children, ..
        } = tree;

        let host_overlay = self.host.as_widget_mut().overlay(
            &mut children[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        // Compute the host's absolute on-screen bounds, accounting for
        // any scroll offset via the translation vector.
        let mut host_bounds = layout.bounds();
        host_bounds.x += translation.x;
        host_bounds.y += translation.y;

        let snackbar_overlay = overlay::Element::new(Box::new(SnackbarOverlay {
            message_text: self.message_text.as_deref(),
            action: self.action.as_ref(),
            on_dismiss: self.on_dismiss.as_ref(),
            anchor: self.anchor,
            severity: self.severity,
            auto_dismiss: self.auto_dismiss,
            state: state.downcast_mut(),
            style_fn: &self.class,
            host_bounds,
            _renderer: std::marker::PhantomData,
        }));

        match host_overlay {
            Some(host) => {
                Some(overlay::Group::with_children(vec![host, snackbar_overlay]).overlay())
            }
            None => Some(snackbar_overlay),
        }
    }
}

/// The overlay rendered when the snackbar is visible.
struct SnackbarOverlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    message_text: Option<&'b str>,
    action: Option<&'b (String, Message)>,
    on_dismiss: Option<&'b Message>,
    anchor: Position,
    severity: Severity,
    auto_dismiss: Option<Duration>,
    state: &'b mut SnackbarState,
    style_fn: &'b <Theme as Catalog>::Class<'a>,
    host_bounds: Rectangle,
    _renderer: std::marker::PhantomData<Renderer>,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for SnackbarOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase,
    Renderer: renderer::Renderer + text::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        let hb = self.host_bounds;

        // Determine bar dimensions.
        let bar_width = (hb.width - EDGE_MARGIN * 2.0).clamp(0.0, MAX_BAR_WIDTH);

        // Positions are relative to the root node (which sits at hb.x, hb.y).
        let bar_x = if self.anchor.is_left() {
            EDGE_MARGIN
        } else if self.anchor.is_right() {
            hb.width - EDGE_MARGIN - bar_width
        } else {
            // Center horizontally
            (hb.width - bar_width) / 2.0
        };

        let bar_y = if self.anchor.is_top() {
            EDGE_MARGIN
        } else if self.anchor.is_bottom() {
            hb.height - EDGE_MARGIN - BAR_HEIGHT
        } else {
            // Center vertically
            (hb.height - BAR_HEIGHT) / 2.0
        };

        let bar_node =
            layout::Node::new(Size::new(bar_width, BAR_HEIGHT)).move_to(Point::new(bar_x, bar_y));

        // Root node covers the host area at its absolute position.
        layout::Node::with_children(Size::new(hb.width, hb.height), vec![bar_node])
            .move_to(Point::new(hb.x, hb.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let snack_style = <Theme as Catalog>::style(theme, self.style_fn, self.severity);
        let bar_layout = layout.children().next().unwrap();
        let bar_bounds = bar_layout.bounds();
        let font_size = theme.text_size();

        // Bar background
        renderer.fill_quad(
            renderer::Quad {
                bounds: bar_bounds,
                border: snack_style.border,
                shadow: snack_style.shadow,
                ..renderer::Quad::default()
            },
            snack_style.background,
        );

        // Severity left border
        if let Some(severity_color) = snack_style.severity_color {
            let border_bounds = Rectangle {
                x: bar_bounds.x,
                y: bar_bounds.y,
                width: SEVERITY_BORDER_WIDTH,
                height: bar_bounds.height,
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds: border_bounds,
                    border: iced::Border {
                        radius: iced::border::Radius {
                            top_left: snack_style.border.radius.top_left,
                            top_right: 0.0,
                            bottom_right: 0.0,
                            bottom_left: snack_style.border.radius.bottom_left,
                        },
                        ..iced::Border::default()
                    },
                    ..renderer::Quad::default()
                },
                iced::Background::Color(severity_color),
            );
        }

        // Calculate content start X (after severity border + icon area)
        let has_severity = snack_style.severity_color.is_some();
        let content_start_x = bar_bounds.x
            + if has_severity {
                SEVERITY_BORDER_WIDTH + INNER_PADDING
            } else {
                INNER_PADDING
            };

        // Severity icon
        if has_severity {
            let icon_glyph = self.severity.fallback_glyph();
            if !icon_glyph.is_empty() {
                let icon_x = content_start_x;
                renderer.fill_text(
                    Text {
                        content: icon_glyph.to_string(),
                        bounds: Size::new(ICON_AREA_WIDTH, BAR_HEIGHT),
                        size: Pixels(font_size),
                        line_height: text::LineHeight::Relative(1.0),
                        font: renderer.default_font(),
                        align_x: iced::alignment::Horizontal::Left.into(),
                        align_y: iced::alignment::Vertical::Center,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(icon_x, bar_bounds.y + BAR_HEIGHT / 2.0),
                    snack_style.severity_color.unwrap(),
                    bar_bounds,
                );
            }
        }

        let text_start_x = content_start_x + if has_severity { ICON_AREA_WIDTH } else { 0.0 };

        // Message text (left-aligned, vertically centered)
        if let Some(msg) = self.message_text {
            let available_width = bar_bounds.x + bar_bounds.width
                - text_start_x
                - INNER_PADDING
                - self.action_width()
                - self.dismiss_width();

            renderer.fill_text(
                Text {
                    content: msg.to_string(),
                    bounds: Size::new(available_width.max(0.0), BAR_HEIGHT),
                    size: Pixels(font_size * 0.875),
                    line_height: text::LineHeight::Relative(1.0),
                    font: renderer.default_font(),
                    align_x: iced::alignment::Horizontal::Left.into(),
                    align_y: iced::alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                },
                Point::new(text_start_x, bar_bounds.y + BAR_HEIGHT / 2.0),
                snack_style.text_color,
                bar_bounds,
            );
        }

        // Action button (right side)
        if let Some((label, _)) = self.action {
            let btn_width = self.action_width();
            let btn_x =
                bar_bounds.x + bar_bounds.width - INNER_PADDING - self.dismiss_width() - btn_width;

            renderer.fill_text(
                Text {
                    content: label.clone(),
                    bounds: Size::new(btn_width, BAR_HEIGHT),
                    size: Pixels(font_size * 0.875),
                    line_height: text::LineHeight::Relative(1.0),
                    font: renderer.default_font(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                },
                Point::new(btn_x + btn_width / 2.0, bar_bounds.y + BAR_HEIGHT / 2.0),
                snack_style.action_color,
                bar_bounds,
            );
        }

        // Dismiss area: countdown or close icon
        if self.on_dismiss.is_some() {
            let dismiss_x = bar_bounds.x + bar_bounds.width - INNER_PADDING - self.dismiss_width();
            let dismiss_center_x = dismiss_x + self.dismiss_width() / 2.0;

            // If auto_dismiss is active and NOT hovering the countdown, show countdown text.
            // Otherwise show close icon.
            let show_countdown = self.auto_dismiss.is_some()
                && self.state.countdown_start.is_some()
                && !self.state.dismiss_hovered;

            if show_countdown {
                let duration = self.auto_dismiss.unwrap();
                let elapsed = self
                    .state
                    .countdown_start
                    .map(|s| Instant::now().duration_since(s))
                    .unwrap_or_default();
                let remaining = duration.saturating_sub(elapsed);
                let secs = remaining.as_secs() + 1; // Round up
                let countdown_text = format!("{secs}s");

                renderer.fill_text(
                    Text {
                        content: countdown_text,
                        bounds: Size::new(self.dismiss_width(), BAR_HEIGHT),
                        size: Pixels(font_size * 0.75),
                        line_height: text::LineHeight::Relative(1.0),
                        font: renderer.default_font(),
                        align_x: iced::alignment::Horizontal::Center.into(),
                        align_y: iced::alignment::Vertical::Center,
                        shaping: text::Shaping::Basic,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(dismiss_center_x, bar_bounds.y + BAR_HEIGHT / 2.0),
                    Color {
                        a: 0.5,
                        ..snack_style.text_color
                    },
                    bar_bounds,
                );
            } else {
                // Close icon (✕)
                renderer.fill_text(
                    Text {
                        content: "\u{2715}".to_string(),
                        bounds: Size::new(24.0, BAR_HEIGHT),
                        size: Pixels(font_size),
                        line_height: text::LineHeight::Relative(1.0),
                        font: renderer.default_font(),
                        align_x: iced::alignment::Horizontal::Center.into(),
                        align_y: iced::alignment::Vertical::Center,
                        shaping: text::Shaping::Basic,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(dismiss_center_x, bar_bounds.y + BAR_HEIGHT / 2.0),
                    snack_style.text_color,
                    bar_bounds,
                );
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
        let bar_layout = layout.children().next().unwrap();
        let bar_bounds = bar_layout.bounds();

        let action_bounds = self.action.as_ref().map(|(_, _)| {
            let btn_width = self.action_width();
            let btn_x =
                bar_bounds.x + bar_bounds.width - INNER_PADDING - self.dismiss_width() - btn_width;
            Rectangle {
                x: btn_x,
                y: bar_bounds.y,
                width: btn_width,
                height: BAR_HEIGHT,
            }
        });

        let dismiss_bounds = if self.on_dismiss.is_some() {
            let dismiss_x = bar_bounds.x + bar_bounds.width - INNER_PADDING - self.dismiss_width();
            Some(Rectangle {
                x: dismiss_x,
                y: bar_bounds.y,
                width: self.dismiss_width(),
                height: BAR_HEIGHT,
            })
        } else {
            None
        };

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.state.action_hovered =
                    action_bounds.map(|b| cursor.is_over(b)).unwrap_or(false);
                self.state.dismiss_hovered =
                    dismiss_bounds.map(|b| cursor.is_over(b)).unwrap_or(false);

                // Request redraw when hovering dismiss area changes (to swap countdown/icon).
                if self.auto_dismiss.is_some() {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if self.state.action_hovered {
                    self.state.action_pressed = true;
                } else if self.state.dismiss_hovered {
                    self.state.dismiss_pressed = true;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.state.action_pressed && self.state.action_hovered {
                    if let Some((_, msg)) = self.action {
                        shell.publish(msg.clone());
                    }
                } else if self.state.dismiss_pressed
                    && self.state.dismiss_hovered
                    && let Some(msg) = self.on_dismiss
                {
                    shell.publish(msg.clone());
                }
                self.state.action_pressed = false;
                self.state.dismiss_pressed = false;
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.state.action_hovered || self.state.dismiss_hovered {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<Message, Theme, Renderer> SnackbarOverlay<'_, '_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    fn action_width(&self) -> f32 {
        self.action
            .as_ref()
            .map(|(label, _)| (label.len() as f32 * 9.0).max(48.0))
            .unwrap_or(0.0)
    }

    fn dismiss_width(&self) -> f32 {
        if self.on_dismiss.is_some() { 36.0 } else { 0.0 }
    }
}

impl<'a, Message, Theme, Renderer> From<Snackbar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn from(snackbar: Snackbar<'a, Message, Theme, Renderer>) -> Self {
        Element::new(snackbar)
    }
}
