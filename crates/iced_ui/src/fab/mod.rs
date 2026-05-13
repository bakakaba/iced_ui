//! Prominent button for the primary action on a screen.
//!
//! # Sizes
//!
//! - **Small** (40x40): compact placement in tight layouts.
//! - **Regular** (56x56): the standard FAB size (default).
//! - **Large** (96x96): emphasized primary action.
//! - **Extended**: adds a text label alongside the icon.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::fab::Fab;
//!
//! let fab = Fab::new(text("+")).on_press(Message::Add);
//! let extended = Fab::new(text("+")).label("New item").on_press(Message::Add);
//! ```

mod style;

pub use style::{Catalog, FabSize, Status, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size};

use crate::{RoundnessBase, SpacingBase};

/// The height of an extended FAB.
const EXTENDED_HEIGHT: f32 = 56.0;

/// Internal state tracking press/hover.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    is_hovered: bool,
    is_pressed: bool,
}

/// A Floating Action Button.
pub struct Fab<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    icon: Element<'a, Message, Theme, Renderer>,
    label: Option<Element<'a, Message, Theme, Renderer>>,
    on_press: Option<Message>,
    fab_size: FabSize,
    lowered: bool,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Fab<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new FAB with the given icon content.
    pub fn new(icon: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            icon: icon.into(),
            label: None,
            on_press: None,
            fab_size: FabSize::default(),
            lowered: false,
            class: Theme::default(),
        }
    }

    /// Sets the size variant.
    pub fn size(mut self, size: FabSize) -> Self {
        self.fab_size = size;
        self
    }

    /// Converts this FAB into an extended FAB by adding a text label.
    pub fn label(mut self, label: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Reduces the elevation (shadow) of the FAB.
    pub fn lowered(mut self) -> Self {
        self.lowered = true;
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

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    fn is_extended(&self) -> bool {
        self.label.is_some()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Fab<'a, Message, Theme, Renderer>
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
        let mut children = vec![Tree::new(&self.icon)];
        if let Some(ref label) = self.label {
            children.push(Tree::new(label));
        }
        children
    }

    fn diff(&self, tree: &mut Tree) {
        let mut refs: Vec<&Element<'_, Message, Theme, Renderer>> = vec![&self.icon];
        if let Some(ref label) = self.label {
            refs.push(label);
        }
        tree.diff_children(&refs);
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
        if self.is_extended() {
            // Extended FAB: icon + 12px gap + label, height 56px, horizontal padding 16px
            let height = EXTENDED_HEIGHT;
            let h_padding = 16.0_f32;
            let gap = 12.0_f32;

            let icon_limits = layout::Limits::new(Size::ZERO, Size::new(24.0, 24.0));
            let mut icon_node =
                self.icon
                    .as_widget_mut()
                    .layout(&mut tree.children[0], renderer, &icon_limits);

            let label_limits = layout::Limits::new(
                Size::ZERO,
                Size::new(limits.max().width - h_padding * 2.0 - 24.0 - gap, height),
            );
            let mut label_node = self.label.as_mut().unwrap().as_widget_mut().layout(
                &mut tree.children[1],
                renderer,
                &label_limits,
            );

            let total_width =
                h_padding + icon_node.size().width + gap + label_node.size().width + h_padding;

            // Position icon centered vertically.
            let icon_y = (height - icon_node.size().height) / 2.0;
            icon_node = icon_node.move_to(iced::Point::new(h_padding, icon_y));

            // Position label centered vertically, after icon + gap.
            let label_x = h_padding + icon_node.size().width + gap;
            let label_y = (height - label_node.size().height) / 2.0;
            label_node = label_node.move_to(iced::Point::new(label_x, label_y));

            layout::Node::with_children(Size::new(total_width, height), vec![icon_node, label_node])
        } else {
            // Standard FAB: square
            let side = self.fab_size.pixels();
            let icon_limits = layout::Limits::new(
                Size::ZERO,
                Size::new(self.fab_size.icon_size(), self.fab_size.icon_size()),
            );
            let mut icon_node =
                self.icon
                    .as_widget_mut()
                    .layout(&mut tree.children[0], renderer, &icon_limits);

            // Center icon in the square.
            let icon_size = icon_node.size();
            icon_node = icon_node.move_to(iced::Point::new(
                (side - icon_size.width) / 2.0,
                (side - icon_size.height) / 2.0,
            ));

            layout::Node::with_children(Size::new(side, side), vec![icon_node])
        }
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

        let status = if state.is_pressed {
            Status::Pressed
        } else if state.is_hovered {
            Status::Hovered
        } else {
            Status::Active
        };

        let fab_style = Catalog::style(theme, &self.class, self.lowered, status);

        // Draw background.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: fab_style.border,
                shadow: fab_style.shadow,
                ..renderer::Quad::default()
            },
            fab_style.background,
        );

        // Draw children.
        let text_style = renderer::Style {
            text_color: fab_style.icon_color,
        };

        let mut children = layout.children();
        let icon_layout = children.next().unwrap();
        self.icon.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &text_style,
            icon_layout,
            cursor,
            viewport,
        );

        if let (Some(label), Some(label_layout)) = (&self.label, children.next()) {
            label.as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                &text_style,
                label_layout,
                cursor,
                viewport,
            );
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
        // Forward to children.
        let mut children = layout.children();
        let icon_layout = children.next().unwrap();
        self.icon.as_widget_mut().update(
            &mut tree.children[0],
            event,
            icon_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if let (Some(label), Some(label_layout)) = (&mut self.label, children.next()) {
            label.as_widget_mut().update(
                &mut tree.children[1],
                event,
                label_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
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
        if self.on_press.is_some() && cursor.is_over(layout.bounds()) {
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
        let mut children = layout.children();
        self.icon.as_widget_mut().operate(
            &mut tree.children[0],
            children.next().unwrap(),
            renderer,
            operation,
        );
        if let (Some(label), Some(label_layout)) = (&mut self.label, children.next()) {
            label
                .as_widget_mut()
                .operate(&mut tree.children[1], label_layout, renderer, operation);
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Fab<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(fab: Fab<'a, Message, Theme, Renderer>) -> Self {
        Element::new(fab)
    }
}
