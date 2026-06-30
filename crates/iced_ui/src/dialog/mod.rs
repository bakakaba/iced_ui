//! Modal overlay presenting an optional title, arbitrary content, and
//! optional action buttons on top of a scrim.
//!
//! The [`Dialog`] widget wraps "host" content. When `open` is `true`,
//! a semi-transparent scrim covers the host, and a centered dialog
//! card is rendered via the overlay system.
//!
//! Unlike a fixed title/body/buttons layout, the dialog card is
//! composed entirely of real widgets: the [`content`](Dialog::content)
//! is any [`Element`], an optional [`title`](Dialog::title) is any
//! [`Element`] placed above it, and the action row is built from real
//! [`Button`](crate::Button)s. Specifying
//! [`confirm`](Dialog::confirm)/[`dismiss`](Dialog::dismiss) generates a
//! standard Material-style "OK"/"Cancel" row; for full control supply a
//! custom [`actions`](Dialog::actions) element, or compose everything
//! inside [`content`](Dialog::content) alone.
//!
//! # Example
//!
//! ```ignore
//! use iced::widget::text;
//! use iced_ui::dialog::Dialog;
//!
//! let dialog = Dialog::new(host_content)
//!     .title(text("Confirm action"))
//!     .content(text("Are you sure you want to proceed?"))
//!     .confirm(Message::Confirmed)
//!     .dismiss(Message::Dismissed)
//!     .open(self.dialog_visible);
//! ```

mod style;

pub use style::{Catalog, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::text;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::widget::{Space, column, row, text as itext};
use iced::{Alignment, Element, Event, Length, Point, Rectangle, Size, Vector, mouse};

use crate::button::{self, Button};
use crate::{FontSizeBase, Roundness, RoundnessBase, SpacingBase};

/// Maximum width of the dialog container.
const MAX_WIDTH: f32 = 560.0;
/// Minimum width of the dialog container.
const MIN_WIDTH: f32 = 280.0;
/// Inner padding of the dialog card, in logical pixels.
const PADDING: f32 = 24.0;
/// Vertical gap between title, content, and the action row.
const SPACING: f32 = 16.0;

/// Trait bounds shared by the dialog's [`Theme`] across every `impl`.
///
/// The dialog composes real [`Button`](crate::Button) and text widgets,
/// so its theme must satisfy their catalogs in addition to the
/// dialog's own.
pub trait DialogTheme:
    Catalog + button::Catalog + iced::widget::text::Catalog + SpacingBase + RoundnessBase + FontSizeBase
{
}

impl<T> DialogTheme for T where
    T: Catalog
        + button::Catalog
        + iced::widget::text::Catalog
        + SpacingBase
        + RoundnessBase
        + FontSizeBase
{
}

/// Wraps host content and conditionally displays a modal dialog
/// overlay composed of real widgets.
pub struct Dialog<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    host: Element<'a, Message, Theme, Renderer>,
    title: Option<Element<'a, Message, Theme, Renderer>>,
    content: Option<Element<'a, Message, Theme, Renderer>>,
    confirm: Option<Message>,
    dismiss: Option<Message>,
    actions: Option<Element<'a, Message, Theme, Renderer>>,
    /// The composed dialog card content, assembled when the [`Dialog`]
    /// is converted into an [`Element`].
    card: Option<Element<'a, Message, Theme, Renderer>>,
    on_scrim_press: Option<Message>,
    open: bool,
    roundness: Option<Roundness>,
    class: <Theme as Catalog>::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Dialog<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: DialogTheme + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    /// Creates a new dialog wrapping the given host content.
    pub fn new(host: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            host: host.into(),
            title: None,
            content: None,
            confirm: None,
            dismiss: None,
            actions: None,
            card: None,
            on_scrim_press: None,
            open: false,
            roundness: None,
            class: <Theme as Catalog>::default(),
        }
    }

    /// Sets the dialog title element, rendered above the content.
    pub fn title(mut self, title: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the main dialog content element.
    pub fn content(mut self, content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Adds a default "OK" confirm button wired to the given message.
    ///
    /// Generates a standard Material-style action row alongside any
    /// [`dismiss`](Self::dismiss) button. For custom labels or buttons,
    /// use [`actions`](Self::actions) instead.
    pub fn confirm(mut self, message: Message) -> Self {
        self.confirm = Some(message);
        self
    }

    /// Adds a default "Cancel" dismiss button wired to the given
    /// message.
    ///
    /// Generates a standard Material-style action row alongside any
    /// [`confirm`](Self::confirm) button. For custom labels or buttons,
    /// use [`actions`](Self::actions) instead.
    pub fn dismiss(mut self, message: Message) -> Self {
        self.dismiss = Some(message);
        self
    }

    /// Sets a custom action row element, overriding the default
    /// confirm/dismiss buttons.
    pub fn actions(mut self, actions: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.actions = Some(actions.into());
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

    /// Overrides the corner roundness of the dialog card, bypassing the
    /// theme's default for this widget. Accepts a [`Roundness`] token:
    /// [`Roundness::sx`] scales the theme's roundness base,
    /// [`Roundness::px`] sets an absolute radius.
    pub fn roundness(mut self, roundness: Roundness) -> Self {
        self.roundness = Some(roundness);
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<<Theme as Catalog>::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Builds the default Material-style action row from the configured
    /// confirm/dismiss messages, if any. Returns `None` when no default
    /// buttons were requested.
    fn default_actions(&self) -> Option<Element<'a, Message, Theme, Renderer>> {
        if self.confirm.is_none() && self.dismiss.is_none() {
            return None;
        }

        // Right-align the buttons within the card.
        let mut actions = row![Space::new().width(Length::Fill)]
            .spacing(8)
            .width(Length::Fill)
            .align_y(Alignment::Center);

        if let Some(msg) = self.dismiss.clone() {
            actions = actions.push(
                Button::new(itext("Cancel"))
                    .variant(button::Variant::Ghost)
                    .on_press(msg),
            );
        }

        if let Some(msg) = self.confirm.clone() {
            actions = actions.push(
                Button::new(itext("OK"))
                    .variant(button::Variant::Solid)
                    .on_press(msg),
            );
        }

        Some(actions.into())
    }

    /// Assembles the dialog card content from the configured title,
    /// content, and action pieces. Consumes the individual slots.
    fn compose(&mut self) {
        let mut col = column![].spacing(SPACING).width(Length::Fill);

        if let Some(title) = self.title.take() {
            col = col.push(title);
        }

        if let Some(content) = self.content.take() {
            col = col.push(content);
        }

        // Prefer a custom action row; otherwise build the default one.
        let actions = self.actions.take().or_else(|| self.default_actions());

        if let Some(actions) = actions {
            col = col.push(actions);
        }

        self.card = Some(col.padding(PADDING).into());
    }
}

/// State for the dialog overlay interaction.
#[derive(Debug, Default)]
struct DialogState {
    /// Whether a press began on the scrim (outside the dialog card).
    scrim_pressed: bool,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Dialog<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: DialogTheme + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DialogState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DialogState::default())
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = vec![Tree::new(&self.host)];
        if let Some(card) = &self.card {
            children.push(Tree::new(card));
        } else {
            children.push(Tree::empty());
        }
        children
    }

    fn diff(&self, tree: &mut Tree) {
        tree.children.resize_with(2, Tree::empty);
        tree.children[0].diff(&self.host);
        if let Some(card) = &self.card {
            tree.children[1].diff(card);
        }
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

        let card = self.card.as_mut()?;
        let (_, card_tree) = tree.children.split_at_mut(1);

        Some(overlay::Element::new(Box::new(DialogOverlay {
            card,
            card_tree: &mut card_tree[0],
            on_scrim_press: self.on_scrim_press.as_ref(),
            state: tree.state.downcast_mut(),
            style_fn: &self.class,
            roundness: self.roundness,
            viewport: *viewport,
        })))
    }
}

/// The overlay rendered when the dialog is open.
struct DialogOverlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    card: &'b mut Element<'a, Message, Theme, Renderer>,
    card_tree: &'b mut Tree,
    on_scrim_press: Option<&'b Message>,
    state: &'b mut DialogState,
    style_fn: &'b <Theme as Catalog>::Class<'a>,
    roundness: Option<Roundness>,
    viewport: Rectangle,
}

impl<Message, Theme, Renderer> DialogOverlay<'_, '_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Computes the centered dialog card rectangle within the viewport.
    fn card_bounds(&self, content: Size) -> Rectangle {
        let viewport_size = self.viewport.size();
        let max_w = MAX_WIDTH.min(viewport_size.width - 48.0).max(MIN_WIDTH);
        let width = content.width.clamp(MIN_WIDTH.min(max_w), max_w);
        let height = content.height.min(viewport_size.height - 48.0);
        let x = (viewport_size.width - width) / 2.0;
        let y = (viewport_size.height - height) / 2.0;
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for DialogOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: DialogTheme,
    Renderer: renderer::Renderer + text::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, _bounds: Size) -> layout::Node {
        let viewport_size = self.viewport.size();
        let max_w = MAX_WIDTH.min(viewport_size.width - 48.0).max(MIN_WIDTH);

        // Measure the composed card content within the available width.
        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(max_w, (viewport_size.height - 48.0).max(0.0)),
        );
        let card_node = self
            .card
            .as_widget_mut()
            .layout(self.card_tree, renderer, &limits);
        let content = card_node.size();

        let bounds = self.card_bounds(content);
        let card_node = card_node.move_to(Point::new(bounds.x, bounds.y));

        layout::Node::with_children(viewport_size, vec![card_node])
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let dialog_style = <Theme as Catalog>::style(theme, self.style_fn);
        let bounds = layout.bounds();

        // Resolve the dialog card border, applying the roundness
        // override when one is set.
        let mut card_border = dialog_style.border;
        if let Some(r) = self.roundness {
            card_border.radius = r.resolve(RoundnessBase::roundness(theme)).into();
        }

        // Scrim
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..renderer::Quad::default()
            },
            iced::Background::Color(dialog_style.scrim_color),
        );

        // Dialog card
        let card_layout = layout.children().next().unwrap();
        let card_bounds = card_layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds: card_bounds,
                border: card_border,
                shadow: dialog_style.shadow,
                ..renderer::Quad::default()
            },
            dialog_style.background,
        );

        // Composed card content (real widgets).
        self.card.as_widget().draw(
            self.card_tree,
            renderer,
            theme,
            style,
            card_layout,
            cursor,
            &card_bounds,
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();
        let card_layout = layout.children().next().unwrap();
        let card_bounds = card_layout.bounds();

        // Forward events to the composed card content first so the
        // real action buttons handle hover/press/clicks.
        self.card.as_widget_mut().update(
            self.card_tree,
            event,
            card_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &card_bounds,
        );

        if shell.is_event_captured() {
            return;
        }

        // Scrim press handling (clicks outside the card).
        let is_over_card = cursor.is_over(card_bounds);
        let is_over_scrim = cursor.is_over(bounds) && !is_over_card;

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if is_over_scrim {
                    self.state.scrim_pressed = true;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.state.scrim_pressed
                    && is_over_scrim
                    && let Some(msg) = self.on_scrim_press
                {
                    shell.publish(msg.clone());
                }
                self.state.scrim_pressed = false;
            }
            _ => {}
        }
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        let card_layout = layout.children().next().unwrap();
        self.card
            .as_widget_mut()
            .operate(self.card_tree, card_layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let card_layout = layout.children().next().unwrap();
        let card_bounds = card_layout.bounds();

        let child = self.card.as_widget().mouse_interaction(
            self.card_tree,
            card_layout,
            cursor,
            &card_bounds,
            renderer,
        );
        if child != mouse::Interaction::default() {
            return child;
        }

        // Pointer over the scrim (dismiss affordance).
        if cursor.is_over(layout.bounds()) && !cursor.is_over(card_bounds) {
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
    Theme: DialogTheme + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn from(mut dialog: Dialog<'a, Message, Theme, Renderer>) -> Self {
        dialog.compose();
        Element::new(dialog)
    }
}
