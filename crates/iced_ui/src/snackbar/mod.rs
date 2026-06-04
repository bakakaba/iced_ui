//! Temporary notification bar at the
//! bottom of its host content.
//!
//! The [`Snackbar`] widget wraps "host" content. When `visible` is
//! `true`, a floating bar is rendered at the bottom via the overlay
//! system, showing a message and optional action button.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::snackbar::Snackbar;
//!
//! let snackbar = Snackbar::new(host_content)
//!     .message("Item deleted")
//!     .action("Undo", Message::Undo)
//!     .on_dismiss(Message::DismissSnackbar)
//!     .visible(self.snackbar_visible);
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
use iced::{Element, Event, Length, Pixels, Point, Rectangle, Size, Vector};

use crate::{FontSizeBase, RoundnessBase, SpacingBase};

/// Height of the snackbar bar in logical pixels.
const BAR_HEIGHT: f32 = 48.0;
/// Horizontal margin from the edges of the host.
const HORIZONTAL_MARGIN: f32 = 16.0;
/// Bottom margin from the host bottom edge.
const BOTTOM_MARGIN: f32 = 16.0;
/// Internal horizontal padding within the bar.
const INNER_PADDING: f32 = 16.0;

/// Wraps host content and conditionally displays a floating
/// notification bar at the bottom with a message and optional action.
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

        Some(overlay::Element::new(Box::new(SnackbarOverlay {
            message_text: self.message_text.as_deref(),
            action: self.action.as_ref(),
            on_dismiss: self.on_dismiss.as_ref(),
            state: tree.state.downcast_mut(),
            style_fn: &self.class,
            viewport: *viewport,
            _renderer: std::marker::PhantomData,
        })))
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
    state: &'b mut SnackbarState,
    style_fn: &'b <Theme as Catalog>::Class<'a>,
    viewport: Rectangle,
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
        let viewport_size = self.viewport.size();

        let bar_width = (viewport_size.width - HORIZONTAL_MARGIN * 2.0).max(0.0);
        let bar_x = HORIZONTAL_MARGIN;
        let bar_y = viewport_size.height - BOTTOM_MARGIN - BAR_HEIGHT;

        let bar_node =
            layout::Node::new(Size::new(bar_width, BAR_HEIGHT)).move_to(Point::new(bar_x, bar_y));

        layout::Node::with_children(viewport_size, vec![bar_node])
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let snack_style = <Theme as Catalog>::style(theme, self.style_fn);
        let bar_layout = layout.children().next().unwrap();
        let bar_bounds = bar_layout.bounds();

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

        // Message text (left-aligned, vertically centered)
        if let Some(msg) = self.message_text {
            let text_x = bar_bounds.x + INNER_PADDING;
            let available_width =
                bar_bounds.width - INNER_PADDING * 2.0 - self.action_width() - self.dismiss_width();

            renderer.fill_text(
                Text {
                    content: msg.to_string(),
                    bounds: Size::new(available_width.max(0.0), BAR_HEIGHT),
                    size: Pixels(theme.text_size() * 0.875),
                    line_height: text::LineHeight::Relative(1.0),
                    font: renderer.default_font(),
                    align_x: iced::alignment::Horizontal::Left.into(),
                    align_y: iced::alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                },
                Point::new(text_x, bar_bounds.y + BAR_HEIGHT / 2.0),
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
                    size: Pixels(theme.text_size() * 0.875),
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

        // Dismiss "X" button (far right)
        if self.on_dismiss.is_some() {
            let dismiss_x = bar_bounds.x + bar_bounds.width - INNER_PADDING - self.dismiss_width();

            renderer.fill_text(
                Text {
                    content: "\u{2715}".to_string(), // ✕
                    bounds: Size::new(24.0, BAR_HEIGHT),
                    size: Pixels(theme.text_size()),
                    line_height: text::LineHeight::Relative(1.0),
                    font: renderer.default_font(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                },
                Point::new(dismiss_x + 12.0, bar_bounds.y + BAR_HEIGHT / 2.0),
                snack_style.text_color,
                bar_bounds,
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
                width: 24.0,
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
        if self.on_dismiss.is_some() { 32.0 } else { 0.0 }
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
