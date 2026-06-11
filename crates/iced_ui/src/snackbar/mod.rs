//! Multi-notification snackbar overlaid on host content.
//!
//! The [`Snackbar`] widget wraps "host" content and displays a stack
//! of active [`Notification`] bars via the overlay system. State is
//! managed by the [`Notifications`] struct which the consumer stores
//! in their application state.
//!
//! # Features
//!
//! - **Multiple notifications** — displays a stack of notifications,
//!   each with its own severity, actions, and auto-dismiss timer.
//! - **Anchor** — position the stack at any corner or edge via
//!   [`Position`].
//! - **Severity** — color-coded left border + icon per notification.
//! - **Auto-dismiss** — per-notification countdown timer that only
//!   starts when the notification becomes visible.
//! - **Actions** — each notification can have multiple text-label
//!   action buttons.
//! - **Managed state** — [`Notifications`] handles the full lifecycle
//!   including capacity limits and pruning of dismissed entries.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::snackbar::{Notification, Notifications, Severity, Snackbar};
//! use iced_ui::Position;
//! use std::time::Duration;
//!
//! // In app state:
//! struct App {
//!     notifications: Notifications,
//! }
//!
//! // In update:
//! self.notifications.push(
//!     Notification::new("File saved")
//!         .severity(Severity::Success)
//!         .auto_dismiss(Duration::from_secs(5))
//! );
//!
//! // In view:
//! Snackbar::new(host_content)
//!     .notifications(&self.notifications)
//!     .on_dismiss(Message::Dismissed)
//!     .on_action(Message::ActionClicked)
//!     .anchor(Position::BottomRight)
//!     .max(5)
//! ```

mod notification;
mod state;
mod style;

pub use notification::{Notification, NotificationId, Severity};
pub use state::Notifications;
pub use style::{Catalog, Style, StyleFn, default};

use std::cell::Cell;
use std::time::Instant;

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
use crate::{FontSizeBase, RoundnessBase, Space, SpacingBase};

/// Height of each notification bar in logical pixels.
const BAR_HEIGHT: f32 = 48.0;
/// Internal horizontal padding within a bar.
const INNER_PADDING: f32 = 16.0;
/// Width of the severity accent border on the left.
const SEVERITY_BORDER_WIDTH: f32 = 4.0;
/// Width reserved for the severity icon area.
const ICON_AREA_WIDTH: f32 = 28.0;
/// Maximum bar width.
const MAX_BAR_WIDTH: f32 = 560.0;
/// Width of the dismiss button area.
const DISMISS_WIDTH: f32 = 36.0;

/// Returns the close icon content and font.
#[cfg(feature = "lucide-icons")]
fn close_icon() -> (String, iced::Font) {
    use crate::icons::FONT;
    (char::from(lucide_icons::Icon::X).to_string(), FONT)
}

/// Returns the close icon content and font (Unicode fallback).
#[cfg(not(feature = "lucide-icons"))]
fn close_icon() -> (String, iced::Font) {
    ("\u{00D7}".to_string(), iced::Font::default())
}

/// Wraps host content and displays a stack of notification bars.
pub struct Snackbar<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    host: Element<'a, Message, Theme, Renderer>,
    notifications: &'a Notifications,
    on_dismiss: Option<Box<dyn Fn(NotificationId) -> Message + 'a>>,
    on_action: Option<Box<dyn Fn(NotificationId, usize) -> Message + 'a>>,
    anchor: Position,
    max: Option<usize>,
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
            notifications: &EMPTY_NOTIFICATIONS,
            on_dismiss: None,
            on_action: None,
            anchor: Position::BottomRight,
            max: None,
            class: Theme::default(),
        }
    }

    /// Sets the notification state to read from.
    ///
    /// The widget displays active (non-dismissed) notifications from
    /// this store. The newest active notifications (by insertion order)
    /// are rendered closest to the anchor edge.
    pub fn notifications(mut self, notifications: &'a Notifications) -> Self {
        self.notifications = notifications;
        self
    }

    /// Sets the callback invoked when a notification is dismissed.
    ///
    /// The callback receives the [`NotificationId`] of the dismissed
    /// notification. The consumer should call
    /// [`Notifications::dismiss()`] with this ID.
    pub fn on_dismiss(mut self, f: impl Fn(NotificationId) -> Message + 'a) -> Self {
        self.on_dismiss = Some(Box::new(f));
        self
    }

    /// Sets the callback invoked when a notification action is clicked.
    ///
    /// The callback receives the [`NotificationId`] and the zero-based
    /// index of the action button that was clicked.
    pub fn on_action(mut self, f: impl Fn(NotificationId, usize) -> Message + 'a) -> Self {
        self.on_action = Some(Box::new(f));
        self
    }

    /// Sets the anchor position for the notification stack.
    /// Defaults to [`Position::BottomRight`].
    pub fn anchor(mut self, position: Position) -> Self {
        self.anchor = position;
        self
    }

    /// Sets the maximum number of notifications displayed at once.
    ///
    /// When more active notifications exist than this limit, only the
    /// newest are shown. Auto-dismiss timers only start when a
    /// notification becomes visible. By default there is no limit.
    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Returns the active notifications that should be displayed,
    /// respecting the `max` limit. Newest last in the returned vec.
    fn visible_notifications(&self) -> Vec<&Notification> {
        let active = self.notifications.active();
        match self.max {
            Some(max) if active.len() > max => active[active.len() - max..].to_vec(),
            _ => active,
        }
    }
}

/// Static empty `Notifications` used as the default when none is provided.
static EMPTY_NOTIFICATIONS: Notifications = Notifications::empty();

/// Internal state for the snackbar widget.
#[derive(Debug)]
struct SnackbarState {
    /// Per-notification auto-dismiss timers: (id, start_instant).
    timers: Vec<(NotificationId, Instant)>,
    /// ID of the notification whose dismiss area is currently hovered.
    dismiss_hovered: Option<NotificationId>,
    /// ID of the notification whose dismiss area is currently pressed.
    dismiss_pressed: Option<NotificationId>,
    /// (notification_id, action_index) of the hovered action button.
    action_hovered: Option<(NotificationId, usize)>,
    /// (notification_id, action_index) of the pressed action button.
    action_pressed: Option<(NotificationId, usize)>,
    /// Cached theme spacing base. Updated in overlay draw() each frame.
    /// Uses `Cell` for interior mutability (draw takes `&self`).
    spacing: Cell<u8>,
}

impl Default for SnackbarState {
    fn default() -> Self {
        Self {
            timers: Vec::new(),
            dismiss_hovered: None,
            dismiss_pressed: None,
            action_hovered: None,
            action_pressed: None,
            spacing: Cell::new(crate::Theme::DEFAULT_SPACING),
        }
    }
}

impl SnackbarState {
    /// Returns the timer start instant for the given notification ID.
    fn timer_for(&self, id: &NotificationId) -> Option<Instant> {
        self.timers
            .iter()
            .find(|(tid, _)| tid == id)
            .map(|(_, t)| *t)
    }

    /// Removes timer entries for IDs not in the given visible set.
    fn cleanup_timers(&mut self, visible_ids: &[&NotificationId]) {
        self.timers.retain(|(id, _)| visible_ids.contains(&id));
    }

    /// Ensures a timer exists for the given ID; returns true if newly added.
    fn ensure_timer(&mut self, id: &NotificationId, now: Instant) -> bool {
        if self.timers.iter().any(|(tid, _)| tid == id) {
            false
        } else {
            self.timers.push((id.clone(), now));
            true
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Snackbar<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = iced::Font> + 'a,
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

        // Manage auto-dismiss timers — only for VISIBLE notifications.
        let visible = self.visible_notifications();
        if visible.is_empty() {
            return;
        }

        let state = tree.state.downcast_mut::<SnackbarState>();

        // Cleanup timers for notifications no longer visible.
        let visible_ids: Vec<&NotificationId> = visible.iter().map(|n| &n.id).collect();
        state.cleanup_timers(&visible_ids);

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let mut needs_redraw = false;

            for notif in &visible {
                if let Some(duration) = notif.auto_dismiss {
                    state.ensure_timer(&notif.id, *now);

                    if let Some(start) = state.timer_for(&notif.id) {
                        let elapsed = now.duration_since(start);
                        if elapsed >= duration {
                            if let Some(ref on_dismiss) = self.on_dismiss {
                                shell.publish(on_dismiss(notif.id.clone()));
                            }
                        } else {
                            needs_redraw = true;
                        }
                    }
                }
            }

            if needs_redraw {
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
        // Compute visible notifications inline to avoid borrowing `self`.
        let active: Vec<&Notification> = self.notifications.active().into_iter().collect();
        let visible: Vec<&Notification> = match self.max {
            Some(max) if active.len() > max => active[active.len() - max..].to_vec(),
            _ => active,
        };

        if visible.is_empty() {
            return self.host.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            );
        }

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

        // Compute the host's absolute on-screen bounds.
        let mut host_bounds = layout.bounds();
        host_bounds.x += translation.x;
        host_bounds.y += translation.y;

        let snackbar_overlay = overlay::Element::new(Box::new(SnackbarOverlay {
            notifications: visible,
            on_dismiss: self.on_dismiss.as_deref(),
            on_action: self.on_action.as_deref(),
            anchor: self.anchor,
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

/// The overlay rendered when notifications are visible.
struct SnackbarOverlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    notifications: Vec<&'b Notification>,
    on_dismiss: Option<&'b (dyn Fn(NotificationId) -> Message + 'a)>,
    on_action: Option<&'b (dyn Fn(NotificationId, usize) -> Message + 'a)>,
    anchor: Position,
    state: &'b mut SnackbarState,
    style_fn: &'b <Theme as Catalog>::Class<'a>,
    host_bounds: Rectangle,
    _renderer: std::marker::PhantomData<Renderer>,
}

impl<Message, Theme, Renderer> SnackbarOverlay<'_, '_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Returns the width needed for action buttons in a notification.
    fn actions_width(notif: &Notification) -> f32 {
        if notif.actions.is_empty() {
            return 0.0;
        }
        notif
            .actions
            .iter()
            .map(|label| (label.len() as f32 * 8.0).max(40.0))
            .sum::<f32>()
            + (notif.actions.len() as f32 - 1.0).max(0.0) * 8.0
    }

    /// Recomputes dismiss/action hover state from the current cursor
    /// position and notification bar layouts.
    fn update_hover_from_cursor(&mut self, cursor: mouse::Cursor, bar_layouts: &[Layout<'_>]) {
        self.state.dismiss_hovered = None;
        self.state.action_hovered = None;

        for (notif, bar_layout) in self.notifications.iter().rev().zip(bar_layouts.iter()) {
            let bar_bounds = bar_layout.bounds();

            // Dismiss hit area
            if self.on_dismiss.is_some() {
                let dismiss_rect = Rectangle {
                    x: bar_bounds.x + bar_bounds.width - INNER_PADDING - DISMISS_WIDTH,
                    y: bar_bounds.y,
                    width: DISMISS_WIDTH,
                    height: BAR_HEIGHT,
                };
                if cursor.is_over(dismiss_rect) {
                    self.state.dismiss_hovered = Some(notif.id.clone());
                }
            }

            // Action hit areas
            if self.on_action.is_some() && !notif.actions.is_empty() {
                let dismiss_w = if self.on_dismiss.is_some() {
                    DISMISS_WIDTH
                } else {
                    0.0
                };
                let actions_w = Self::actions_width(notif);
                let mut action_x =
                    bar_bounds.x + bar_bounds.width - INNER_PADDING - dismiss_w - actions_w;

                for (i, label) in notif.actions.iter().enumerate() {
                    let btn_width = (label.len() as f32 * 8.0).max(40.0);
                    let action_rect = Rectangle {
                        x: action_x,
                        y: bar_bounds.y,
                        width: btn_width,
                        height: BAR_HEIGHT,
                    };
                    if cursor.is_over(action_rect) {
                        self.state.action_hovered = Some((notif.id.clone(), i));
                    }
                    action_x += btn_width + 8.0;
                }
            }
        }
    }

    /// Computes the bar width and x position for the notification stack.
    fn bar_geometry(&self) -> (f32, f32) {
        let hb = self.host_bounds;
        let edge_margin = Space::sx(2.0).resolve(self.state.spacing.get());
        let bar_width = (hb.width - edge_margin * 2.0).clamp(0.0, MAX_BAR_WIDTH);

        let bar_x = if self.anchor.is_left() {
            edge_margin
        } else if self.anchor.is_right() {
            hb.width - edge_margin - bar_width
        } else {
            (hb.width - bar_width) / 2.0
        };

        (bar_width, bar_x)
    }

    /// Computes the bounding rectangle for the notification stack area
    /// (relative to the host bounds origin).
    fn stack_bounds(&self) -> Rectangle {
        let (bar_width, bar_x) = self.bar_geometry();
        let edge_margin = Space::sx(2.0).resolve(self.state.spacing.get());
        let stack_gap = Space::sx(1.0).resolve(self.state.spacing.get());
        let n = self.notifications.len();
        let stack_height =
            BAR_HEIGHT * n as f32 + stack_gap * (n as f32 - 1.0).max(0.0) + edge_margin;

        let stack_y = if self.anchor.is_bottom() {
            self.host_bounds.height - stack_height
        } else {
            edge_margin
        };

        Rectangle {
            x: bar_x,
            y: stack_y,
            width: bar_width,
            height: stack_height,
        }
    }
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for SnackbarOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase,
    Renderer: renderer::Renderer + text::Renderer<Font = iced::Font>,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        let hb = self.host_bounds;
        let (bar_width, _) = self.bar_geometry();
        let stack = self.stack_bounds();
        let edge_margin = Space::sx(2.0).resolve(self.state.spacing.get());
        let stack_gap = Space::sx(1.0).resolve(self.state.spacing.get());

        // Stack notifications: newest (last in vec) is closest to anchor edge.
        // Positions are RELATIVE to the stack root node.
        let mut children = Vec::with_capacity(self.notifications.len());
        let mut offset = edge_margin;

        for _notif in self.notifications.iter().rev() {
            let bar_y = if self.anchor.is_bottom() {
                stack.height - offset - BAR_HEIGHT
            } else {
                offset - edge_margin
            };

            offset += BAR_HEIGHT + stack_gap;

            children.push(
                layout::Node::new(Size::new(bar_width, BAR_HEIGHT)).move_to(Point::new(0.0, bar_y)),
            );
        }

        // Root node only covers the notification stack area (not the full host).
        layout::Node::with_children(Size::new(stack.width, stack.height), children)
            .move_to(Point::new(hb.x + stack.x, hb.y + stack.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        // Cache the theme spacing for use in layout() on subsequent frames.
        self.state.spacing.set(theme.spacing());

        let font_size = theme.text_size();

        for (notif, bar_layout) in self.notifications.iter().rev().zip(layout.children()) {
            let bar_bounds = bar_layout.bounds();
            let snack_style = <Theme as Catalog>::style(theme, self.style_fn, notif.severity);

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

            // Content layout
            let has_severity = snack_style.severity_color.is_some();
            let content_start_x = bar_bounds.x
                + if has_severity {
                    SEVERITY_BORDER_WIDTH + INNER_PADDING
                } else {
                    INNER_PADDING
                };

            // Severity icon
            if has_severity && let Some((icon_content, icon_font)) = notif.severity.icon_content() {
                renderer.fill_text(
                    Text {
                        content: icon_content,
                        bounds: Size::new(ICON_AREA_WIDTH, BAR_HEIGHT),
                        size: Pixels(font_size),
                        line_height: text::LineHeight::Relative(1.0),
                        font: icon_font,
                        align_x: iced::alignment::Horizontal::Left.into(),
                        align_y: iced::alignment::Vertical::Center,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(content_start_x, bar_bounds.y + BAR_HEIGHT / 2.0),
                    snack_style.severity_color.unwrap(),
                    bar_bounds,
                );
            }

            let text_start_x = content_start_x + if has_severity { ICON_AREA_WIDTH } else { 0.0 };

            // Dismiss area width
            let dismiss_w = if self.on_dismiss.is_some() {
                DISMISS_WIDTH
            } else {
                0.0
            };

            // Actions width
            let actions_w = Self::actions_width(notif);

            // Message text
            let available_width = bar_bounds.x + bar_bounds.width
                - text_start_x
                - INNER_PADDING
                - actions_w
                - dismiss_w;

            renderer.fill_text(
                Text {
                    content: notif.message.clone(),
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

            // Action buttons (right side, before dismiss)
            if !notif.actions.is_empty() {
                let mut action_x =
                    bar_bounds.x + bar_bounds.width - INNER_PADDING - dismiss_w - actions_w;

                for label in &notif.actions {
                    let btn_width = (label.len() as f32 * 8.0).max(40.0);

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
                        Point::new(action_x + btn_width / 2.0, bar_bounds.y + BAR_HEIGHT / 2.0),
                        snack_style.action_color,
                        bar_bounds,
                    );

                    action_x += btn_width + 8.0;
                }
            }

            // Dismiss area: countdown or close icon
            if self.on_dismiss.is_some() {
                let dismiss_x = bar_bounds.x + bar_bounds.width - INNER_PADDING - DISMISS_WIDTH;
                let dismiss_center_x = dismiss_x + DISMISS_WIDTH / 2.0;

                // Show countdown if auto_dismiss is active and NOT hovering
                let show_countdown = notif.auto_dismiss.is_some()
                    && self.state.timer_for(&notif.id).is_some()
                    && self.state.dismiss_hovered.as_ref() != Some(&notif.id);

                if show_countdown {
                    let duration = notif.auto_dismiss.unwrap();
                    let elapsed = self
                        .state
                        .timer_for(&notif.id)
                        .map(|s| Instant::now().duration_since(s))
                        .unwrap_or_default();
                    let remaining = duration.saturating_sub(elapsed);
                    let secs = remaining.as_secs() + 1;
                    let countdown_text = format!("{secs}s");

                    renderer.fill_text(
                        Text {
                            content: countdown_text,
                            bounds: Size::new(DISMISS_WIDTH, BAR_HEIGHT),
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
                    // Close icon
                    let (close_content, close_font) = close_icon();
                    renderer.fill_text(
                        Text {
                            content: close_content,
                            bounds: Size::new(24.0, BAR_HEIGHT),
                            size: Pixels(font_size),
                            line_height: text::LineHeight::Relative(1.0),
                            font: close_font,
                            align_x: iced::alignment::Horizontal::Center.into(),
                            align_y: iced::alignment::Vertical::Center,
                            shaping: text::Shaping::Advanced,
                            wrapping: text::Wrapping::None,
                        },
                        Point::new(dismiss_center_x, bar_bounds.y + BAR_HEIGHT / 2.0),
                        snack_style.text_color,
                        bar_bounds,
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
        let bar_layouts: Vec<_> = layout.children().collect();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                // Recompute hover state from current cursor position.
                // Done on both CursorMoved AND ButtonPressed to handle the
                // case where the layout shifted (e.g. a notification was
                // dismissed) without the cursor moving.
                self.update_hover_from_cursor(cursor, &bar_layouts);

                if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) {
                    if self.state.dismiss_hovered.is_some() {
                        self.state.dismiss_pressed = self.state.dismiss_hovered.clone();
                    } else if self.state.action_hovered.is_some() {
                        self.state.action_pressed = self.state.action_hovered.clone();
                    }
                }

                // Request redraw for countdown hover state change.
                if self.notifications.iter().any(|n| n.auto_dismiss.is_some()) {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                // Dismiss click
                if let Some(ref pressed_id) = self.state.dismiss_pressed
                    && self.state.dismiss_hovered.as_ref() == Some(pressed_id)
                    && let Some(ref on_dismiss) = self.on_dismiss
                {
                    shell.publish(on_dismiss(pressed_id.clone()));
                }
                // Action click
                if let Some((ref pressed_id, pressed_idx)) = self.state.action_pressed
                    && self.state.action_hovered.as_ref()
                        == Some(&(pressed_id.clone(), pressed_idx))
                    && let Some(ref on_action) = self.on_action
                {
                    shell.publish(on_action(pressed_id.clone(), pressed_idx));
                }
                self.state.dismiss_pressed = None;
                self.state.action_pressed = None;
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
        if self.state.dismiss_hovered.is_some() || self.state.action_hovered.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Snackbar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = iced::Font> + 'a,
{
    fn from(snackbar: Snackbar<'a, Message, Theme, Renderer>) -> Self {
        Element::new(snackbar)
    }
}
