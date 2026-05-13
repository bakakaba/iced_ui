//! Circular/rounded button showing a single icon, with four visual
//! variants and an optional toggle state.
//!
//! # Variants
//!
//! - **Standard**: no fill, icon inherits surface text color.
//! - **Filled**: primary-colored background.
//! - **Filled Tonal**: secondary/tonal background.
//! - **Outlined**: surface background with a visible border.
//!
//! Each variant supports a `toggled` state for use as a stateful
//! toggle button.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::icon_button::{IconButton, Variant};
//!
//! let btn = IconButton::new(text("X"))
//!     .variant(Variant::Filled)
//!     .on_press(Message::Close);
//! ```

mod style;

pub use style::{Catalog, Status, Style, StyleFn, Variant, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Padding, Rectangle, Size};

use crate::{RoundnessBase, SpacingBase};

/// Default icon button size (logical pixels).
const DEFAULT_SIZE: f32 = 40.0;

/// Internal state tracking press/hover.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    is_hovered: bool,
    is_pressed: bool,
}

/// A Material Design 3 icon button.
pub struct IconButton<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<Message>,
    variant: Variant,
    toggled: bool,
    size: f32,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> IconButton<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new icon button wrapping the given icon content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            on_press: None,
            variant: Variant::default(),
            toggled: false,
            size: DEFAULT_SIZE,
            class: Theme::default(),
        }
    }

    /// Sets the visual variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the toggle state.
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }

    /// Sets the button size (width and height) in logical pixels.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the message emitted when the button is pressed.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the message emitted when pressed, if `Some`.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    fn is_enabled(&self) -> bool {
        self.on_press.is_some()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for IconButton<'a, Message, Theme, Renderer>
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
        tree.diff_children(std::slice::from_ref(&self.content));
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
        let size = Size::new(self.size, self.size);
        let _limits = limits
            .width(Length::Fixed(self.size))
            .height(Length::Fixed(self.size));

        let content_limits =
            layout::Limits::new(Size::ZERO, size).shrink(Padding::new(self.size * 0.2));

        let mut content_node =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &content_limits);

        // Center the content inside the button.
        let content_size = content_node.size();
        content_node = content_node.move_to(iced::Point::new(
            (size.width - content_size.width) / 2.0,
            (size.height - content_size.height) / 2.0,
        ));

        layout::Node::with_children(size, vec![content_node])
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

        let status = if !self.is_enabled() {
            Status::Disabled
        } else if state.is_pressed {
            Status::Pressed
        } else if state.is_hovered {
            Status::Hovered
        } else {
            Status::Active
        };

        let button_style = Catalog::style(theme, &self.class, self.variant, status, self.toggled);

        // Draw the button background.
        if button_style.background.is_some() || button_style.border.width > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: button_style.border,
                    shadow: button_style.shadow,
                    ..renderer::Quad::default()
                },
                button_style
                    .background
                    .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
            );
        }

        // Draw the icon content.
        let content_layout = layout.children().next().unwrap();

        // Apply icon color as text color via a modified style.
        let text_style = renderer::Style {
            text_color: button_style.icon_color,
        };
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
        // Forward events to the content child.
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if !self.is_enabled() {
            return;
        }

        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let is_over = cursor.is_over(bounds);

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                state.is_hovered = is_over;
                if !is_over {
                    state.is_pressed = false;
                }
            }
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
            _ => {}
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
        if self.is_enabled() && cursor.is_over(layout.bounds()) {
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
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<IconButton<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(button: IconButton<'a, Message, Theme, Renderer>) -> Self {
        Element::new(button)
    }
}
