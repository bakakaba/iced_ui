//! A horizontal application menu bar with dropdowns, nested submenus,
//! keyboard shortcuts and a pluggable style.
//!
//! See [`MenuBar`] for the widget, [`Item`] and [`Menu`] for its
//! entries, and [`shortcuts`] plus [`KeyBinding`] for key-binding
//! support.

mod item;
mod overlay;
mod shortcut;
mod style;

use std::cell::Cell;
use std::marker::PhantomData;

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay as advanced_overlay;
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph, Text};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::alignment;
use iced::mouse;
use iced::{
    Border, Color, Element, Event, Font, Length, Padding, Pixels, Point, Rectangle, Size, Vector,
};

pub use item::{Entry, Icon, Item, Menu, Separator};
pub use shortcut::{KeyBinding, shortcuts};
pub use style::{Catalog, Style, StyleFn, default};

use item::collect_shortcuts;
use overlay::{MenuOverlay, Metrics};

/// Cached state for a single bar label.
#[derive(Debug, Default)]
struct BarLabel<P: Paragraph> {
    paragraph: P,
    content: String,
}

/// Internal widget state stored in the widget tree.
#[derive(Debug)]
pub(crate) struct State<P: Paragraph> {
    /// Per top-level menu cached paragraph.
    bar_labels: Vec<BarLabel<P>>,
    /// Path of currently-open menus. `None` means closed.
    pub(crate) open_path: Option<Vec<usize>>,
    /// Index of the top-level bar label that is visually "active"
    /// (currently open or pressed).
    pub(crate) bar_active: Option<usize>,
    /// Index of the top-level bar label currently under the cursor
    /// (when the bar is closed). Tracked in state so that `draw` can
    /// render hover deterministically and `update` can request redraws
    /// only when the hovered label changes.
    pub(crate) bar_hover: Option<usize>,
    /// Rows of menus that the cursor is currently hovering, per depth.
    /// Length matches `open_path.len()` when open.
    pub(crate) hover_path: Vec<Option<usize>>,
    /// Last-resolved `Style::spacing`, cached so that hooks that do not
    /// have access to `&Theme` (layout, update, mouse_interaction,
    /// overlay) can stay consistent with the most recent draw. Kept in
    /// a [`Cell`] so `draw`, which only sees `&State`, can refresh it.
    pub(crate) spacing_cache: Cell<f32>,
}

impl<P: Paragraph> Default for State<P> {
    fn default() -> Self {
        Self {
            bar_labels: Vec::new(),
            open_path: None,
            bar_active: None,
            bar_hover: None,
            hover_path: Vec::new(),
            spacing_cache: Cell::new(8.0),
        }
    }
}

impl<P: Paragraph> State<P> {
    pub(crate) fn close(&mut self) {
        self.open_path = None;
        self.hover_path.clear();
    }
}

/// A horizontal application menu bar.
///
/// See the [module-level docs](crate::menu) for an overview, and
/// [`MenuBar::shortcuts`] for how to wire up keyboard shortcuts.
pub struct MenuBar<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    menus: Vec<Menu<Message>>,
    width: Length,
    padding: Padding,
    text_size: Option<Pixels>,
    font: Option<Font>,
    class: <Theme as Catalog>::Class<'a>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer> MenuBar<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    /// Creates an empty [`MenuBar`].
    pub fn new() -> Self {
        Self {
            menus: Vec::new(),
            width: Length::Fill,
            padding: Padding::new(4.0),
            text_size: None,
            font: None,
            class: <Theme as Catalog>::default(),
            _renderer: PhantomData,
        }
    }

    /// Creates a [`MenuBar`] pre-populated with the given top-level menus.
    pub fn with_menus(menus: Vec<Menu<Message>>) -> Self {
        Self {
            menus,
            ..Self::new()
        }
    }

    /// Appends a top-level [`Menu`] to this bar.
    pub fn push(mut self, menu: Menu<Message>) -> Self {
        self.menus.push(menu);
        self
    }

    /// Sets the width of the bar. Defaults to [`Length::Fill`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the outer padding around the bar's row of labels.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size used for bar labels and menu items.
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = Some(size.into());
        self
    }

    /// Sets the font used for bar labels and menu items.
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }
}

impl<'a, Message, Renderer> MenuBar<'a, Message, crate::Theme, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer<Font = Font>,
{
    /// Sets the style of the bar using a `Fn(&Theme) -> Style` closure.
    ///
    /// This is available when `Theme = `[`crate::Theme`].
    pub fn style(mut self, style: impl Fn(&crate::Theme) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }
}

impl<Message: Clone, Theme, Renderer> MenuBar<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    /// Returns the list of all `(KeyBinding, Message)` pairs declared
    /// on items within this bar (including inside nested submenus).
    ///
    /// Pass the returned vec to [`shortcuts`] to produce a
    /// [`Subscription`](iced::Subscription) that dispatches the
    /// matching message whenever the user presses one of the
    /// combinations.
    pub fn shortcuts(&self) -> Vec<(KeyBinding, Message)> {
        collect_shortcuts(&self.menus)
    }
}

impl<Message, Theme, Renderer> Default for MenuBar<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for MenuBar<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size()).0;
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let spacing = state.spacing_cache.get();
        let metrics = Metrics::new(text_size, self.padding, spacing);

        // Resize cached bar-label paragraphs to match.
        if state.bar_labels.len() != self.menus.len() {
            state
                .bar_labels
                .resize_with(self.menus.len(), BarLabel::default);
        }

        for (slot, menu) in state.bar_labels.iter_mut().zip(&self.menus) {
            if slot.content != menu.label {
                slot.content = menu.label.clone();
                slot.paragraph = Renderer::Paragraph::with_text(Text {
                    content: &slot.content,
                    bounds: Size::new(f32::INFINITY, f32::INFINITY),
                    size: Pixels(text_size),
                    line_height: text::LineHeight::default(),
                    font,
                    align_x: text::Alignment::Default,
                    align_y: alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
                });
            }
        }

        let bar_height = metrics.row_height;

        let max_width = limits.max().width;
        let width = match self.width {
            Length::Shrink => {
                let mut w = self.padding.left + self.padding.right;
                let n = state.bar_labels.len();
                for slot in &state.bar_labels {
                    w += slot.paragraph.min_width() + self.padding.left + self.padding.right;
                }
                if n > 1 {
                    w += spacing * (n as f32 - 1.0);
                }
                w.min(max_width)
            }
            _ => max_width,
        };

        layout::Node::new(Size::new(width, bar_height))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let style = theme.style(&self.class);
        state.spacing_cache.set(style.spacing);
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.bar_border,
                ..renderer::Quad::default()
            },
            style.bar_background,
        );

        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size()).0;
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        let mut x = bounds.x + self.padding.left;
        let last_index = state.bar_labels.len().saturating_sub(1);
        for (i, slot) in state.bar_labels.iter().enumerate() {
            let w = slot.paragraph.min_width();
            let label_bounds = Rectangle {
                x,
                y: bounds.y,
                width: w + self.padding.left + self.padding.right,
                height: bounds.height,
            };

            let is_active = state.bar_active == Some(i);
            let is_hovered = state.bar_hover == Some(i);

            let (bg, text_color) = if is_active || is_hovered {
                (
                    Some(style.bar_item_background_active),
                    style.bar_text_active,
                )
            } else {
                (None, style.bar_text)
            };

            if let Some(bg) = bg {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: label_bounds,
                        border: Border {
                            radius: style.item_radius.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        ..renderer::Quad::default()
                    },
                    bg,
                );
            }

            renderer.fill_text(
                Text {
                    content: slot.content.clone(),
                    bounds: Size::new(w, text_size),
                    size: Pixels(text_size),
                    line_height: text::LineHeight::default(),
                    font,
                    align_x: text::Alignment::Default,
                    align_y: alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
                },
                Point::new(
                    label_bounds.x + self.padding.left,
                    label_bounds.y + (label_bounds.height - text_size) / 2.0,
                ),
                text_color,
                *viewport,
            );

            x += label_bounds.width;
            if i < last_index {
                x += style.spacing;
            }
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();
        let spacing = state.spacing_cache.get();
        let label_bounds = compute_bar_label_bounds(state, bounds, self.padding, spacing);

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let new_hover = cursor
                    .position()
                    .and_then(|pos| label_bounds.iter().position(|label| label.contains(pos)));
                if state.bar_hover != new_hover {
                    state.bar_hover = new_hover;
                    shell.request_redraw();
                }
                return;
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                if state.bar_hover.is_some() {
                    state.bar_hover = None;
                    shell.request_redraw();
                }
                return;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {}
            _ => return,
        }

        let Some(pos) = cursor.position() else {
            return;
        };

        for (i, label) in label_bounds.iter().enumerate() {
            if label.contains(pos) {
                if state.open_path == Some(vec![i]) {
                    // Toggle off.
                    state.close();
                    state.bar_active = None;
                } else {
                    state.open_path = Some(vec![i]);
                    state.bar_active = Some(i);
                    state.hover_path = vec![Some(i)];
                }
                shell.capture_event();
                shell.request_redraw();
                return;
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();
        let spacing = state.spacing_cache.get();
        let label_bounds = compute_bar_label_bounds(state, bounds, self.padding, spacing);

        let Some(pos) = cursor.position() else {
            return mouse::Interaction::None;
        };

        for label in &label_bounds {
            if label.contains(pos) {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::None
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<advanced_overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        state.open_path.as_ref()?;

        let bar_bounds = layout.bounds() + translation;
        let spacing = state.spacing_cache.get();
        let bar_label_bounds = compute_bar_label_bounds(state, bar_bounds, self.padding, spacing);

        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size()).0;
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let metrics = Metrics::new(text_size, self.padding, spacing);

        let overlay = MenuOverlay {
            menus: &mut self.menus,
            state,
            bar_bounds,
            bar_label_bounds,
            metrics,
            font,
            style_fn: &self.class,
        };

        Some(advanced_overlay::Element::new(Box::new(overlay)))
    }
}

fn compute_bar_label_bounds<P: Paragraph>(
    state: &State<P>,
    bar_bounds: Rectangle,
    padding: Padding,
    spacing: f32,
) -> Vec<Rectangle> {
    let mut out = Vec::with_capacity(state.bar_labels.len());
    let mut x = bar_bounds.x + padding.left;
    for (i, slot) in state.bar_labels.iter().enumerate() {
        let w = slot.paragraph.min_width() + padding.left + padding.right;
        out.push(Rectangle {
            x,
            y: bar_bounds.y,
            width: w,
            height: bar_bounds.height,
        });
        x += w;
        if i + 1 < state.bar_labels.len() {
            x += spacing;
        }
    }
    out
}

impl<'a, Message, Theme, Renderer> From<MenuBar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer<Font = Font> + 'a,
{
    fn from(bar: MenuBar<'a, Message, Theme, Renderer>) -> Self {
        Element::new(bar)
    }
}
