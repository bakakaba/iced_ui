//! Modal overlay presenting a title, content, and action buttons
//! on top of a scrim.
//!
//! The [`Dialog`] widget wraps "host" content. When `open` is `true`,
//! a semi-transparent scrim covers the host, and a centered dialog
//! card is rendered via the overlay system.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::dialog::Dialog;
//!
//! let dialog = Dialog::new(host_content)
//!     .title("Confirm action")
//!     .body("Are you sure you want to proceed?")
//!     .confirm("OK", Message::Confirmed)
//!     .dismiss("Cancel", Message::Dismissed)
//!     .open(self.dialog_visible);
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

/// Maximum width of the dialog container.
const MAX_WIDTH: f32 = 560.0;
/// Minimum width of the dialog container.
const MIN_WIDTH: f32 = 280.0;

/// Wraps host content and conditionally displays a modal dialog
/// overlay with title, body text, and action buttons.
pub struct Dialog<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    host: Element<'a, Message, Theme, Renderer>,
    title: Option<String>,
    body: Option<String>,
    confirm: Option<(String, Message)>,
    dismiss: Option<(String, Message)>,
    on_scrim_press: Option<Message>,
    open: bool,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Dialog<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new dialog wrapping the given host content.
    pub fn new(host: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            host: host.into(),
            title: None,
            body: None,
            confirm: None,
            dismiss: None,
            on_scrim_press: None,
            open: false,
            class: Theme::default(),
        }
    }

    /// Sets the dialog title text.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the dialog body text.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Adds a confirm action button with a label and message.
    pub fn confirm(mut self, label: impl Into<String>, message: Message) -> Self {
        self.confirm = Some((label.into(), message));
        self
    }

    /// Adds a dismiss action button with a label and message.
    pub fn dismiss(mut self, label: impl Into<String>, message: Message) -> Self {
        self.dismiss = Some((label.into(), message));
        self
    }

    /// Sets the message emitted when the scrim is pressed (typically
    /// to close the dialog).
    pub fn on_scrim_press(mut self, message: Message) -> Self {
        self.on_scrim_press = Some(message);
        self
    }

    /// Controls whether the dialog is visible.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// State for the dialog overlay interaction.
#[derive(Debug, Default)]
struct DialogState {
    confirm_hovered: bool,
    confirm_pressed: bool,
    dismiss_hovered: bool,
    dismiss_pressed: bool,
    scrim_pressed: bool,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Dialog<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DialogState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DialogState::default())
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
        if !self.open {
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
        if self.open {
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
        if !self.open {
            return self.host.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            );
        }

        Some(overlay::Element::new(Box::new(DialogOverlay {
            title: self.title.as_deref(),
            body: self.body.as_deref(),
            confirm: self.confirm.as_ref(),
            dismiss: self.dismiss.as_ref(),
            on_scrim_press: self.on_scrim_press.as_ref(),
            state: tree.state.downcast_mut(),
            style_fn: &self.class,
            viewport: *viewport,
            _renderer: std::marker::PhantomData,
        })))
    }
}

/// The overlay rendered when the dialog is open.
struct DialogOverlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    title: Option<&'b str>,
    body: Option<&'b str>,
    confirm: Option<&'b (String, Message)>,
    dismiss: Option<&'b (String, Message)>,
    on_scrim_press: Option<&'b Message>,
    state: &'b mut DialogState,
    style_fn: &'b <Theme as Catalog>::Class<'a>,
    viewport: Rectangle,
    _renderer: std::marker::PhantomData<Renderer>,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for DialogOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog + SpacingBase + RoundnessBase,
    Renderer: renderer::Renderer + text::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        let viewport_size = self.viewport.size();

        let dialog_width = MAX_WIDTH.min(viewport_size.width - 48.0).max(MIN_WIDTH);
        let padding = 24.0_f32;
        let mut content_height = padding;

        if self.title.is_some() {
            content_height += 28.0 + 16.0;
        }

        if self.body.is_some() {
            content_height += 60.0 + 24.0; // estimate body height
        }

        if self.confirm.is_some() || self.dismiss.is_some() {
            content_height += 40.0;
        }

        content_height += padding;

        let dialog_height = content_height.min(viewport_size.height - 48.0);
        let dialog_x = (viewport_size.width - dialog_width) / 2.0;
        let dialog_y = (viewport_size.height - dialog_height) / 2.0;

        let dialog_node = layout::Node::new(Size::new(dialog_width, dialog_height))
            .move_to(Point::new(dialog_x, dialog_y));

        layout::Node::with_children(viewport_size, vec![dialog_node])
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let dialog_style = <Theme as Catalog>::style(theme, self.style_fn);
        let bounds = layout.bounds();

        // Scrim
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..renderer::Quad::default()
            },
            iced::Background::Color(dialog_style.scrim_color),
        );

        // Dialog card
        let dialog_layout = layout.children().next().unwrap();
        let dialog_bounds = dialog_layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds: dialog_bounds,
                border: dialog_style.border,
                shadow: dialog_style.shadow,
                ..renderer::Quad::default()
            },
            dialog_style.background,
        );

        let padding = 24.0_f32;
        let mut y_cursor = dialog_bounds.y + padding;

        // Title
        if let Some(title) = self.title {
            renderer.fill_text(
                Text {
                    content: title.to_string(),
                    bounds: Size::new(dialog_bounds.width - padding * 2.0, 28.0),
                    size: Pixels(22.0),
                    line_height: text::LineHeight::Relative(1.2),
                    font: renderer.default_font(),
                    align_x: iced::alignment::Horizontal::Left.into(),
                    align_y: iced::alignment::Vertical::Top,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                },
                Point::new(dialog_bounds.x + padding, y_cursor),
                dialog_style.title_color,
                dialog_bounds,
            );
            y_cursor += 28.0 + 16.0;
        }

        // Body
        if let Some(body) = self.body {
            renderer.fill_text(
                Text {
                    content: body.to_string(),
                    bounds: Size::new(dialog_bounds.width - padding * 2.0, 200.0),
                    size: Pixels(14.0),
                    line_height: text::LineHeight::Relative(1.4),
                    font: renderer.default_font(),
                    align_x: iced::alignment::Horizontal::Left.into(),
                    align_y: iced::alignment::Vertical::Top,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::WordOrGlyph,
                },
                Point::new(dialog_bounds.x + padding, y_cursor),
                dialog_style.text_color,
                dialog_bounds,
            );
        }

        // Action buttons
        if self.confirm.is_some() || self.dismiss.is_some() {
            let button_y = dialog_bounds.y + dialog_bounds.height - padding - 40.0;
            let mut btn_x = dialog_bounds.x + dialog_bounds.width - padding;

            if let Some((label, _)) = self.confirm {
                let btn_width = (label.len() as f32 * 9.0).max(64.0);
                btn_x -= btn_width;

                let bg_color = dialog_style.title_color;

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: btn_x,
                            y: button_y,
                            width: btn_width,
                            height: 40.0,
                        },
                        border: Border {
                            radius: 20.0.into(),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    iced::Background::Color(iced::Color {
                        a: if self.state.confirm_pressed {
                            0.2
                        } else if self.state.confirm_hovered {
                            0.15
                        } else {
                            0.1
                        },
                        ..bg_color
                    }),
                );

                renderer.fill_text(
                    Text {
                        content: label.clone(),
                        bounds: Size::new(btn_width, 40.0),
                        size: Pixels(14.0),
                        line_height: text::LineHeight::Relative(1.0),
                        font: renderer.default_font(),
                        align_x: iced::alignment::Horizontal::Center.into(),
                        align_y: iced::alignment::Vertical::Center,
                        shaping: text::Shaping::Basic,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(btn_x + btn_width / 2.0, button_y + 20.0),
                    dialog_style.title_color,
                    Rectangle {
                        x: btn_x,
                        y: button_y,
                        width: btn_width,
                        height: 40.0,
                    },
                );

                btn_x -= 8.0;
            }

            if let Some((label, _)) = self.dismiss {
                let btn_width = (label.len() as f32 * 9.0).max(64.0);
                btn_x -= btn_width;

                renderer.fill_text(
                    Text {
                        content: label.clone(),
                        bounds: Size::new(btn_width, 40.0),
                        size: Pixels(14.0),
                        line_height: text::LineHeight::Relative(1.0),
                        font: renderer.default_font(),
                        align_x: iced::alignment::Horizontal::Center.into(),
                        align_y: iced::alignment::Vertical::Center,
                        shaping: text::Shaping::Basic,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(btn_x + btn_width / 2.0, button_y + 20.0),
                    dialog_style.title_color,
                    Rectangle {
                        x: btn_x,
                        y: button_y,
                        width: btn_width,
                        height: 40.0,
                    },
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
        let bounds = layout.bounds();
        let dialog_layout = layout.children().next().unwrap();
        let dialog_bounds = dialog_layout.bounds();

        let is_over_dialog = cursor.is_over(dialog_bounds);
        let is_over_scrim = cursor.is_over(bounds) && !is_over_dialog;

        let padding = 24.0_f32;
        let button_y = dialog_bounds.y + dialog_bounds.height - padding - 40.0;
        let mut btn_x = dialog_bounds.x + dialog_bounds.width - padding;

        let confirm_bounds = self.confirm.as_ref().map(|(label, _)| {
            let btn_width = (label.len() as f32 * 9.0).max(64.0);
            btn_x -= btn_width;
            let r = Rectangle {
                x: btn_x,
                y: button_y,
                width: btn_width,
                height: 40.0,
            };
            btn_x -= 8.0;
            r
        });

        let dismiss_bounds = self.dismiss.as_ref().map(|(label, _)| {
            let btn_width = (label.len() as f32 * 9.0).max(64.0);
            btn_x -= btn_width;
            Rectangle {
                x: btn_x,
                y: button_y,
                width: btn_width,
                height: 40.0,
            }
        });

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.state.confirm_hovered =
                    confirm_bounds.map(|b| cursor.is_over(b)).unwrap_or(false);
                self.state.dismiss_hovered =
                    dismiss_bounds.map(|b| cursor.is_over(b)).unwrap_or(false);
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if self.state.confirm_hovered {
                    self.state.confirm_pressed = true;
                } else if self.state.dismiss_hovered {
                    self.state.dismiss_pressed = true;
                } else if is_over_scrim {
                    self.state.scrim_pressed = true;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.state.confirm_pressed && self.state.confirm_hovered {
                    if let Some((_, msg)) = self.confirm {
                        shell.publish(msg.clone());
                    }
                } else if self.state.dismiss_pressed && self.state.dismiss_hovered {
                    if let Some((_, msg)) = self.dismiss {
                        shell.publish(msg.clone());
                    }
                } else if self.state.scrim_pressed
                    && is_over_scrim
                    && let Some(msg) = self.on_scrim_press
                {
                    shell.publish(msg.clone());
                }
                self.state.confirm_pressed = false;
                self.state.dismiss_pressed = false;
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
        let dialog_layout = layout.children().next().unwrap();
        let dialog_bounds = dialog_layout.bounds();

        if self.state.confirm_hovered || self.state.dismiss_hovered {
            mouse::Interaction::Pointer
        } else if cursor.is_over(dialog_bounds) {
            mouse::Interaction::default()
        } else if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Dialog<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn from(dialog: Dialog<'a, Message, Theme, Renderer>) -> Self {
        Element::new(dialog)
    }
}
