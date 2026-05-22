//! A themed button with variant and size options.
//!
//! # Variants
//!
//! - **Solid** (default): filled background with contrasting text.
//! - **Outline**: border outline with no fill; text uses primary color.
//! - **Ghost**: transparent at rest; gains a subtle background on hover.
//!
//! # Sizes
//!
//! - **Sm** (24px), **Md** (32px, default), **Lg** (40px).
//!
//! # Example
//!
//! ```ignore
//! use iced::widget::text;
//! use iced_ui::button::{Button, ButtonSize, Variant};
//!
//! let btn = Button::new(text("Save")).on_press(Message::Save);
//! let outline = Button::new(text("Cancel"))
//!     .variant(Variant::Outline)
//!     .size(ButtonSize::Sm)
//!     .on_press(Message::Cancel);
//! ```

mod style;

pub use style::{ButtonSize, Catalog, Status, Style, StyleFn, Variant, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size, window};

use crate::{RoundnessBase, SpacingBase};

/// Internal state tracking press/hover and last-rendered status.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    is_pressed: bool,
    /// The status that was last rendered; used to detect changes and
    /// request redraws.
    last_status: Option<Status>,
}

/// A themed button widget.
pub struct Button<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    variant: Variant,
    btn_size: ButtonSize,
    on_press: Option<Message>,
    enabled: bool,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Button<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new button with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            variant: Variant::default(),
            btn_size: ButtonSize::default(),
            on_press: None,
            enabled: true,
            class: Theme::default(),
        }
    }

    /// Sets the visual variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.btn_size = size;
        self
    }

    /// Sets the message emitted when pressed.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the message emitted when pressed, if `Some`.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    /// Enables or disables the button.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Returns whether the button is interactive (enabled and has an
    /// on_press handler).
    fn is_interactive(&self) -> bool {
        self.enabled && self.on_press.is_some()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Button<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content]);
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
        let height = self.btn_size.height();
        let h_padding = self.btn_size.h_padding();

        // Layout content within the available space minus padding.
        let content_limits = layout::Limits::new(
            Size::ZERO,
            Size::new(limits.max().width - h_padding * 2.0, height),
        );
        let mut content_node =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &content_limits);

        // Center content vertically within the button height.
        let content_size = content_node.size();
        let content_x = h_padding;
        let content_y = (height - content_size.height) / 2.0;
        content_node = content_node.move_to(iced::Point::new(content_x, content_y));

        let total_width = h_padding + content_size.width + h_padding;

        layout::Node::with_children(Size::new(total_width, height), vec![content_node])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let is_over = cursor.is_over(bounds);

        let status = if !self.enabled {
            Status::Disabled
        } else if state.is_pressed {
            Status::Pressed
        } else if is_over {
            Status::Hovered
        } else {
            Status::Active
        };

        let btn_style = Catalog::style(theme, &self.class, self.variant, self.btn_size, status);

        // Draw background.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: btn_style.border,
                shadow: btn_style.shadow,
                ..renderer::Quad::default()
            },
            btn_style
                .background
                .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
        );

        // Draw content.
        let text_style = renderer::Style {
            text_color: btn_style.text_color,
        };

        let content_layout = layout.children().next().unwrap();
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &text_style,
            content_layout,
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
        // Forward to content child.
        let content_layout = layout.children().next().unwrap();
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let is_over = cursor.is_over(bounds);

        if self.is_interactive() {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if is_over => {
                    state.is_pressed = true;
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    if state.is_pressed
                        && is_over
                        && let Some(msg) = self.on_press.clone()
                    {
                        shell.publish(msg);
                    }
                    state.is_pressed = false;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. }) if !is_over => {
                    state.is_pressed = false;
                }
                _ => {}
            }
        }

        // Compute the current visual status.
        let current_status = if !self.enabled {
            Status::Disabled
        } else if state.is_pressed {
            Status::Pressed
        } else if is_over {
            Status::Hovered
        } else {
            Status::Active
        };

        // Request a redraw when status changes (e.g. hover enter/leave).
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            state.last_status = Some(current_status);
        } else if state.last_status.is_some_and(|s| s != current_status) {
            shell.request_redraw();
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
        if self.is_interactive() && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let content_layout = layout.children().next().unwrap();
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            content_layout,
            renderer,
            operation,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Button<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(button: Button<'a, Message, Theme, Renderer>) -> Self {
        Element::new(button)
    }
}
