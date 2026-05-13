//! Compact interactive element for
//! actions, filtering, or representing input.
//!
//! # Kinds
//!
//! - **Assist**: guides toward a specific action.
//! - **Filter**: toggles a filter in a group.
//! - **Input**: represents a piece of information (e.g. contact).
//! - **Suggestion**: contextual quick action/reply.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::chip::Chip;
//!
//! let chip = Chip::filter("Vegetarian")
//!     .selected(true)
//!     .on_press(Message::ToggleFilter);
//! ```

mod style;

pub use style::{Catalog, Kind, Status, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size};

use crate::{RoundnessBase, SpacingBase};

/// Height of a chip in logical pixels (MD3 spec).
const CHIP_HEIGHT: f32 = 32.0;

/// Internal state.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    is_hovered: bool,
    is_pressed: bool,
}

/// Chip widget.
pub struct Chip<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    label: Element<'a, Message, Theme, Renderer>,
    leading_icon: Option<Element<'a, Message, Theme, Renderer>>,
    trailing_icon: Option<Element<'a, Message, Theme, Renderer>>,
    kind: Kind,
    selected: bool,
    on_press: Option<Message>,
    on_close: Option<Message>,
    enabled: bool,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Chip<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates an assist chip with the given label.
    pub fn assist(label: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self::new(label, Kind::Assist)
    }

    /// Creates a filter chip with the given label.
    pub fn filter(label: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self::new(label, Kind::Filter)
    }

    /// Creates an input chip with the given label.
    pub fn input(label: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self::new(label, Kind::Input)
    }

    /// Creates a suggestion chip with the given label.
    pub fn suggestion(label: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self::new(label, Kind::Suggestion)
    }

    fn new(label: impl Into<Element<'a, Message, Theme, Renderer>>, kind: Kind) -> Self {
        Self {
            label: label.into(),
            leading_icon: None,
            trailing_icon: None,
            kind,
            selected: false,
            on_press: None,
            on_close: None,
            enabled: true,
            class: Theme::default(),
        }
    }

    /// Sets a leading icon element.
    pub fn icon(mut self, icon: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// Sets a trailing icon element (often used for close/remove).
    pub fn trailing_icon(mut self, icon: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.trailing_icon = Some(icon.into());
        self
    }

    /// Sets the selected (toggled) state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets the message emitted when the chip body is pressed.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the message emitted when the chip body is pressed, if
    /// `Some`.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    /// Sets the message emitted when the close/trailing icon is
    /// pressed.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Enables or disables the chip.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    fn child_count(&self) -> usize {
        1 + self.leading_icon.is_some() as usize + self.trailing_icon.is_some() as usize
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Chip<'a, Message, Theme, Renderer>
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
        let mut children = Vec::with_capacity(self.child_count());
        if let Some(ref icon) = self.leading_icon {
            children.push(Tree::new(icon));
        }
        children.push(Tree::new(&self.label));
        if let Some(ref icon) = self.trailing_icon {
            children.push(Tree::new(icon));
        }
        children
    }

    fn diff(&self, tree: &mut Tree) {
        let mut refs: Vec<&Element<'_, Message, Theme, Renderer>> =
            Vec::with_capacity(self.child_count());
        if let Some(ref icon) = self.leading_icon {
            refs.push(icon);
        }
        refs.push(&self.label);
        if let Some(ref icon) = self.trailing_icon {
            refs.push(icon);
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
        let h_padding = 16.0_f32;
        let icon_gap = 8.0_f32;
        let icon_max = 18.0_f32;

        let mut children_nodes = Vec::new();
        let mut x_cursor = h_padding;
        let mut child_idx = 0;

        // Leading icon
        if let Some(ref mut icon) = self.leading_icon {
            let icon_limits = layout::Limits::new(Size::ZERO, Size::new(icon_max, icon_max));
            let mut node =
                icon.as_widget_mut()
                    .layout(&mut tree.children[child_idx], renderer, &icon_limits);
            let icon_y = (CHIP_HEIGHT - node.size().height) / 2.0;
            node = node.move_to(iced::Point::new(x_cursor, icon_y));
            x_cursor += node.size().width + icon_gap;
            children_nodes.push(node);
            child_idx += 1;
        }

        // Label
        let label_limits = layout::Limits::new(
            Size::ZERO,
            Size::new(limits.max().width - x_cursor - h_padding, CHIP_HEIGHT),
        );
        let mut label_node = self.label.as_widget_mut().layout(
            &mut tree.children[child_idx],
            renderer,
            &label_limits,
        );
        let label_y = (CHIP_HEIGHT - label_node.size().height) / 2.0;
        label_node = label_node.move_to(iced::Point::new(x_cursor, label_y));
        x_cursor += label_node.size().width;
        children_nodes.push(label_node);
        child_idx += 1;

        // Trailing icon
        if let Some(ref mut icon) = self.trailing_icon {
            x_cursor += icon_gap;
            let icon_limits = layout::Limits::new(Size::ZERO, Size::new(icon_max, icon_max));
            let mut node =
                icon.as_widget_mut()
                    .layout(&mut tree.children[child_idx], renderer, &icon_limits);
            let icon_y = (CHIP_HEIGHT - node.size().height) / 2.0;
            node = node.move_to(iced::Point::new(x_cursor, icon_y));
            x_cursor += node.size().width;
            children_nodes.push(node);
        }

        x_cursor += h_padding;

        layout::Node::with_children(Size::new(x_cursor, CHIP_HEIGHT), children_nodes)
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

        let status = if !self.enabled {
            Status::Disabled
        } else if state.is_pressed {
            Status::Pressed
        } else if state.is_hovered {
            Status::Hovered
        } else {
            Status::Active
        };

        let chip_style = Catalog::style(theme, &self.class, self.kind, self.selected, status);

        // Draw background.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: chip_style.border,
                shadow: chip_style.shadow,
                ..renderer::Quad::default()
            },
            chip_style
                .background
                .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
        );

        // Draw children.
        let text_style = renderer::Style {
            text_color: chip_style.text_color,
        };

        for (idx, child_layout) in layout.children().enumerate() {
            let child_widget: &dyn Widget<Message, Theme, Renderer> =
                if let Some(lead) = &self.leading_icon {
                    match idx {
                        0 => lead.as_widget(),
                        1 => self.label.as_widget(),
                        _ => self.trailing_icon.as_ref().unwrap().as_widget(),
                    }
                } else {
                    match idx {
                        0 => self.label.as_widget(),
                        _ => self.trailing_icon.as_ref().unwrap().as_widget(),
                    }
                };

            child_widget.draw(
                &tree.children[idx],
                renderer,
                theme,
                &text_style,
                child_layout,
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
        for (child_idx, child_layout) in layout.children().enumerate() {
            if child_idx < tree.children.len() {
                let widget: &mut dyn Widget<Message, Theme, Renderer> =
                    if let Some(lead) = &mut self.leading_icon {
                        match child_idx {
                            0 => lead.as_widget_mut(),
                            1 => self.label.as_widget_mut(),
                            _ => self.trailing_icon.as_mut().unwrap().as_widget_mut(),
                        }
                    } else {
                        match child_idx {
                            0 => self.label.as_widget_mut(),
                            _ => self.trailing_icon.as_mut().unwrap().as_widget_mut(),
                        }
                    };

                widget.update(
                    &mut tree.children[child_idx],
                    event,
                    child_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        }

        if !self.enabled {
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
                if state.is_pressed && is_over {
                    // Check if click is on trailing icon area for on_close
                    if self.on_close.is_some() && self.trailing_icon.is_some() {
                        // Simple heuristic: last 32px is trailing icon area
                        let trailing_x = bounds.x + bounds.width - 32.0;
                        if let Some(pos) = cursor.position()
                            && pos.x >= trailing_x
                            && let Some(msg) = self.on_close.clone()
                        {
                            shell.publish(msg);
                            state.is_pressed = false;
                            return;
                        }
                    }
                    if let Some(msg) = self.on_press.clone() {
                        shell.publish(msg);
                    }
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
        if self.enabled && cursor.is_over(layout.bounds()) {
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
        for (idx, child_layout) in layout.children().enumerate() {
            if idx < tree.children.len() {
                let widget: &mut dyn Widget<Message, Theme, Renderer> =
                    if let Some(lead) = &mut self.leading_icon {
                        match idx {
                            0 => lead.as_widget_mut(),
                            1 => self.label.as_widget_mut(),
                            _ => self.trailing_icon.as_mut().unwrap().as_widget_mut(),
                        }
                    } else {
                        match idx {
                            0 => self.label.as_widget_mut(),
                            _ => self.trailing_icon.as_mut().unwrap().as_widget_mut(),
                        }
                    };

                widget.operate(&mut tree.children[idx], child_layout, renderer, operation);
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Chip<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(chip: Chip<'a, Message, Theme, Renderer>) -> Self {
        Element::new(chip)
    }
}
