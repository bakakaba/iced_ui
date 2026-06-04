//! Panel that slides from the bottom of its host content.
//!
//! The [`BottomSheet`] widget wraps "host" content. When `expanded` is
//! `true`, a sheet panel is rendered at the bottom via the overlay
//! system. Optionally a scrim covers the host when `modal` is `true`.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::bottom_sheet::BottomSheet;
//!
//! let sheet = BottomSheet::new(host_content, "Sheet body text")
//!     .modal(true)
//!     .expanded(self.sheet_expanded)
//!     .on_dismiss(Message::CloseSheet)
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
use iced::{Element, Event, Length, Pixels, Point, Rectangle, Size, Vector};

use crate::{FontSizeBase, RoundnessBase, SpacingBase};

/// Width of the drag handle pill in logical pixels.
const HANDLE_WIDTH: f32 = 32.0;
/// Height of the drag handle pill in logical pixels.
const HANDLE_HEIGHT: f32 = 4.0;
/// Vertical padding above/below the drag handle.
const HANDLE_VERTICAL_PADDING: f32 = 12.0;
/// Inner padding for sheet content.
const SHEET_PADDING: f32 = 24.0;

/// Wraps host content and conditionally displays a sheet panel from
/// the bottom of the viewport.
pub struct BottomSheet<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    host: Element<'a, Message, Theme, Renderer>,
    sheet_body: String,
    modal: bool,
    expanded: bool,
    on_dismiss: Option<Message>,
    drag_handle: bool,
    height_fraction: f32,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> BottomSheet<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new bottom sheet wrapping the given host content with
    /// the provided sheet body text.
    pub fn new(
        host: impl Into<Element<'a, Message, Theme, Renderer>>,
        sheet_body: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            sheet_body: sheet_body.into(),
            modal: false,
            expanded: false,
            on_dismiss: None,
            drag_handle: true,
            height_fraction: 0.5,
            class: Theme::default(),
        }
    }

    /// Controls whether the sheet is modal (shows scrim over host).
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Controls whether the sheet is expanded (visible).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Sets the message emitted when the scrim is pressed to dismiss.
    pub fn on_dismiss(mut self, msg: Message) -> Self {
        self.on_dismiss = Some(msg);
        self
    }

    /// Controls whether the drag handle pill is shown.
    pub fn drag_handle(mut self, show: bool) -> Self {
        self.drag_handle = show;
        self
    }

    /// Sets the sheet height as a fraction of the viewport (0.0–1.0).
    /// Defaults to 0.5 (50%).
    pub fn height_fraction(mut self, fraction: f32) -> Self {
        self.height_fraction = fraction.clamp(0.1, 1.0);
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// State for the bottom sheet overlay interaction.
#[derive(Debug, Default)]
struct SheetState {
    scrim_pressed: bool,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for BottomSheet<'a, Message, Theme, Renderer>
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
        if !self.expanded || !self.modal {
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
        if self.expanded && self.modal {
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
        if !self.expanded {
            return self.host.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            );
        }

        Some(overlay::Element::new(Box::new(SheetOverlay {
            sheet_body: &self.sheet_body,
            modal: self.modal,
            on_dismiss: self.on_dismiss.as_ref(),
            drag_handle: self.drag_handle,
            height_fraction: self.height_fraction,
            state: tree.state.downcast_mut(),
            style_fn: &self.class,
            viewport: *viewport,
            _renderer: std::marker::PhantomData,
        })))
    }
}

/// The overlay rendered when the bottom sheet is expanded.
struct SheetOverlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    sheet_body: &'b str,
    modal: bool,
    on_dismiss: Option<&'b Message>,
    drag_handle: bool,
    height_fraction: f32,
    state: &'b mut SheetState,
    style_fn: &'b <Theme as Catalog>::Class<'a>,
    viewport: Rectangle,
    _renderer: std::marker::PhantomData<Renderer>,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for SheetOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase,
    Renderer: renderer::Renderer + text::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        let viewport_size = self.viewport.size();

        let sheet_height = (viewport_size.height * self.height_fraction).min(viewport_size.height);
        let sheet_y = viewport_size.height - sheet_height;

        let sheet_node = layout::Node::new(Size::new(viewport_size.width, sheet_height))
            .move_to(Point::new(0.0, sheet_y));

        layout::Node::with_children(viewport_size, vec![sheet_node])
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let sheet_style = <Theme as Catalog>::style(theme, self.style_fn);
        let bounds = layout.bounds();

        // Scrim (if modal)
        if self.modal {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..renderer::Quad::default()
                },
                iced::Background::Color(sheet_style.scrim_color),
            );
        }

        // Sheet panel
        let sheet_layout = layout.children().next().unwrap();
        let sheet_bounds = sheet_layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds: sheet_bounds,
                border: sheet_style.border,
                shadow: sheet_style.shadow,
                ..renderer::Quad::default()
            },
            sheet_style.background,
        );

        let mut y_cursor = sheet_bounds.y;

        // Drag handle
        if self.drag_handle {
            y_cursor += HANDLE_VERTICAL_PADDING;
            let handle_x = sheet_bounds.x + (sheet_bounds.width - HANDLE_WIDTH) / 2.0;

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: handle_x,
                        y: y_cursor,
                        width: HANDLE_WIDTH,
                        height: HANDLE_HEIGHT,
                    },
                    border: iced::Border {
                        radius: (HANDLE_HEIGHT / 2.0).into(),
                        ..iced::Border::default()
                    },
                    ..renderer::Quad::default()
                },
                iced::Background::Color(sheet_style.handle_color),
            );

            y_cursor += HANDLE_HEIGHT + HANDLE_VERTICAL_PADDING;
        } else {
            y_cursor += SHEET_PADDING;
        }

        // Body text
        let text_x = sheet_bounds.x + SHEET_PADDING;
        let text_width = sheet_bounds.width - SHEET_PADDING * 2.0;
        let text_height = sheet_bounds.height - (y_cursor - sheet_bounds.y) - SHEET_PADDING;

        renderer.fill_text(
            Text {
                content: self.sheet_body.to_string(),
                bounds: Size::new(text_width.max(0.0), text_height.max(0.0)),
                size: Pixels(theme.text_size() * 0.875),
                line_height: text::LineHeight::Relative(1.4),
                font: renderer.default_font(),
                align_x: iced::alignment::Horizontal::Left.into(),
                align_y: iced::alignment::Vertical::Top,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::WordOrGlyph,
            },
            Point::new(text_x, y_cursor),
            sheet_style.handle_color, // reuse handle color for text (on-surface)
            sheet_bounds,
        );
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
        let sheet_layout = layout.children().next().unwrap();
        let sheet_bounds = sheet_layout.bounds();

        let is_over_sheet = cursor.is_over(sheet_bounds);
        let is_over_scrim = cursor.is_over(bounds) && !is_over_sheet;

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if is_over_scrim && self.modal =>
            {
                self.state.scrim_pressed = true;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.state.scrim_pressed
                    && is_over_scrim
                    && let Some(msg) = self.on_dismiss
                {
                    shell.publish(msg.clone());
                }
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
        let sheet_layout = layout.children().next().unwrap();
        let sheet_bounds = sheet_layout.bounds();

        if cursor.is_over(sheet_bounds) {
            mouse::Interaction::default()
        } else if self.modal && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<BottomSheet<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn from(sheet: BottomSheet<'a, Message, Theme, Renderer>) -> Self {
        Element::new(sheet)
    }
}
