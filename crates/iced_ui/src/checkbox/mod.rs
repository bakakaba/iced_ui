//! A checkbox whose behavior is driven by the type of its value.
//!
//! The value passed to [`Checkbox::new`] decides whether the checkbox
//! is a plain two-state control or a tri-state control that can also
//! represent an indeterminate ("unset") value:
//!
//! - A [`bool`] value produces a binary checkbox. Clicking toggles
//!   `false` ⇄ `true`; the indeterminate state is unreachable.
//! - An [`Option<bool>`] value produces a tri-state checkbox that can
//!   *display* the indeterminate (`None`) value. It has two modes,
//!   selected via [`Checkbox::indeterminate`]:
//!   - **Read-only (default):** the app may set `None`, but a click
//!     only moves between checked and unchecked — the user can never
//!     click *into* `None`. A click while indeterminate selects
//!     (`Some(true)`). Useful for a "select all" parent that reflects
//!     mixed children.
//!   - **Cyclable** (`.indeterminate(true)`): clicking cycles
//!     `None` → `Some(true)` → `Some(false)` → `None`, so the user
//!     can select the indeterminate value, e.g. a tri-state filter.
//!
//! In both cases the closure registered with [`Checkbox::on_toggle`]
//! receives the *next* value, expressed in the same type that was
//! passed to [`Checkbox::new`].
//!
//! The display-facing [`State`] (used by the styling [`Catalog`]) is
//! derived from the value via the [`Value`] trait.
//!
//! When the `lucide-icons` feature is enabled, the check and dash
//! marks are rendered with the bundled Lucide icon font; otherwise the
//! widget falls back to Unicode glyphs drawn with the default font.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::checkbox::Checkbox;
//!
//! // Binary checkbox — `on_toggle` receives a `bool`.
//! let binary = Checkbox::new(true)
//!     .label("Remember me")
//!     .on_toggle(Message::Remember);
//!
//! // Tri-state checkbox, read-only indeterminate (default): the
//! // app can show `None`, but clicking only toggles checked/unchecked.
//! let select_all = Checkbox::new(None)
//!     .label("Select all")
//!     .on_toggle(Message::SelectAll);
//!
//! // Tri-state checkbox, cyclable indeterminate: the user can click
//! // into the `None` ("any") value.
//! let filter = Checkbox::new(None)
//!     .label("Any")
//!     .indeterminate(true)
//!     .on_toggle(Message::Filter);
//! ```

mod style;

pub use style::{Catalog, Status, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, Text, text};
use iced::alignment;
use iced::mouse;
use iced::{Element, Event, Length, Pixels, Point, Rectangle, Size};

use crate::{RoundnessBase, SpacingBase};

/// Default edge length of the checkbox box, in logical pixels (MD3
/// spec uses an 18px target with a larger touch area).
const DEFAULT_BOX_SIZE: f32 = 18.0;

/// The display-facing state of a [`Checkbox`].
///
/// This is derived from the checkbox's value (see [`Value::state`])
/// and drives both the rendered mark and the styling [`Catalog`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum State {
    /// The checkbox is not selected.
    #[default]
    Unchecked,
    /// The checkbox is selected.
    Checked,
    /// The checkbox represents an indeterminate value — either a mix
    /// of selected and unselected children ("select all"), or an
    /// explicit "unset" choice in a tri-state control.
    Indeterminate,
}

impl State {
    /// Returns `true` if the box renders a mark (checked or
    /// indeterminate).
    pub fn is_marked(self) -> bool {
        matches!(self, State::Checked | State::Indeterminate)
    }
}

mod sealed {
    /// Prevents [`Value`](super::Value) from being implemented for
    /// types other than the ones supported by this crate.
    pub trait Sealed {}

    impl Sealed for bool {}
    impl Sealed for Option<bool> {}
}

/// A value type accepted by [`Checkbox::new`].
///
/// The implementing type determines the checkbox's behavior: a
/// [`bool`] yields a binary control, while an [`Option<bool>`] yields
/// a tri-state control that can *display* the indeterminate (`None`)
/// value. Whether the user can click *into* `None` is controlled by
/// [`Checkbox::indeterminate`].
///
/// This trait is sealed and cannot be implemented outside this crate;
/// only [`bool`] and [`Option<bool>`] are valid value types.
pub trait Value: sealed::Sealed + Copy {
    /// Maps this value to its display-facing [`State`].
    fn state(self) -> State;

    /// Returns the value produced by clicking when the indeterminate
    /// state is *read-only* (the default).
    ///
    /// The user may move between checked and unchecked but can never
    /// click *into* the indeterminate value; a click while
    /// indeterminate selects (checked).
    fn toggled(self) -> Self;

    /// Returns the value produced by clicking when the indeterminate
    /// state is *cyclable*.
    ///
    /// The user can reach the indeterminate value by clicking, cycling
    /// indeterminate → checked → unchecked → indeterminate.
    fn cycled(self) -> Self;
}

impl Value for bool {
    fn state(self) -> State {
        if self {
            State::Checked
        } else {
            State::Unchecked
        }
    }

    fn toggled(self) -> Self {
        !self
    }

    fn cycled(self) -> Self {
        // A binary checkbox has no indeterminate state to cycle into,
        // so cycling is identical to toggling.
        !self
    }
}

impl Value for Option<bool> {
    fn state(self) -> State {
        match self {
            Some(true) => State::Checked,
            Some(false) => State::Unchecked,
            None => State::Indeterminate,
        }
    }

    fn toggled(self) -> Self {
        // Read-only indeterminate: a click never returns to `None`. A
        // click while indeterminate (`None`) selects (`Some(true)`);
        // otherwise toggle checked/unchecked.
        match self {
            None | Some(false) => Some(true),
            Some(true) => Some(false),
        }
    }

    fn cycled(self) -> Self {
        // Cyclable indeterminate:
        // Indeterminate -> Checked -> Unchecked -> Indeterminate.
        match self {
            None => Some(true),
            Some(true) => Some(false),
            Some(false) => None,
        }
    }
}

/// Returns the mark glyph content and font for the given state, or
/// `None` if no mark should be drawn.
#[cfg(feature = "lucide-icons")]
fn mark_glyph(state: State) -> Option<(char, iced::Font)> {
    use crate::icons::FONT;
    match state {
        State::Checked => Some((char::from(lucide_icons::Icon::Check), FONT)),
        State::Indeterminate => Some((char::from(lucide_icons::Icon::Minus), FONT)),
        State::Unchecked => None,
    }
}

/// Returns the mark glyph content and font for the given state, or
/// `None` if no mark should be drawn (Unicode fallback).
#[cfg(not(feature = "lucide-icons"))]
fn mark_glyph(state: State) -> Option<(char, iced::Font)> {
    match state {
        State::Checked => Some(('\u{2713}', iced::Font::default())),
        State::Indeterminate => Some(('\u{2212}', iced::Font::default())),
        State::Unchecked => None,
    }
}

/// Internal interaction state.
#[derive(Debug, Clone, Copy, Default)]
struct InternalState {
    is_hovered: bool,
    is_pressed: bool,
}

/// A checkbox widget whose behavior is driven by its value type `V`
/// (see the [module documentation](self) and [`Value`]).
#[allow(missing_debug_implementations)]
pub struct Checkbox<'a, V, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    V: Value,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    value: V,
    label: Option<Element<'a, Message, Theme, Renderer>>,
    on_toggle: Option<Box<dyn Fn(V) -> Message + 'a>>,
    size: f32,
    enabled: bool,
    cycle_indeterminate: bool,
    class: Theme::Class<'a>,
}

impl<'a, V, Message, Theme, Renderer> Checkbox<'a, V, Message, Theme, Renderer>
where
    V: Value,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a checkbox with the given value.
    ///
    /// Passing a [`bool`] yields a binary checkbox; passing an
    /// [`Option<bool>`] yields a tri-state checkbox that can *display*
    /// the indeterminate (`None`) value. By default a click can only
    /// move between checked and unchecked (the indeterminate state is
    /// read-only); call [`Checkbox::indeterminate`] to let the user
    /// cycle into `None`.
    pub fn new(value: V) -> Self {
        Self {
            value,
            label: None,
            on_toggle: None,
            size: DEFAULT_BOX_SIZE,
            enabled: true,
            cycle_indeterminate: false,
            class: Theme::default(),
        }
    }

    /// Sets a label element rendered to the right of the box.
    pub fn label(mut self, label: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the closure invoked when the checkbox is clicked.
    ///
    /// The closure receives the *next* value the checkbox would move
    /// to (in the same type passed to [`Checkbox::new`]) and returns
    /// the message to emit.
    pub fn on_toggle(mut self, f: impl Fn(V) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }

    /// Sets the toggle closure if `Some`, leaving the checkbox
    /// non-interactive otherwise.
    pub fn on_toggle_maybe(mut self, f: Option<impl Fn(V) -> Message + 'a>) -> Self {
        self.on_toggle = f.map(|f| Box::new(f) as Box<dyn Fn(V) -> Message + 'a>);
        self
    }

    /// Sets the edge length of the box, in logical pixels.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Enables or disables the checkbox.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    fn is_interactive(&self) -> bool {
        self.enabled && self.on_toggle.is_some()
    }
}

impl<'a, Message, Theme, Renderer> Checkbox<'a, Option<bool>, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Controls whether the user can click *into* the indeterminate
    /// (`None`) value.
    ///
    /// This builder is only available on tri-state
    /// (`Checkbox<Option<bool>>`) checkboxes; a binary
    /// (`Checkbox<bool>`) checkbox has no indeterminate state, so the
    /// method does not exist for it.
    ///
    /// - `false` (the default) — the indeterminate state is
    ///   *read-only*: the app may still set it, but a click only moves
    ///   between checked and unchecked. A click while indeterminate
    ///   selects (`Some(true)`).
    /// - `true` — the indeterminate state is *cyclable*: clicking
    ///   cycles `None` → `Some(true)` → `Some(false)` → `None`, so the
    ///   user can reach the indeterminate value.
    pub fn indeterminate(mut self, cycle: bool) -> Self {
        self.cycle_indeterminate = cycle;
        self
    }
}

impl<'a, V, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Checkbox<'a, V, Message, Theme, Renderer>
where
    V: Value,
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = iced::Font> + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<InternalState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(InternalState::default())
    }

    fn children(&self) -> Vec<Tree> {
        match &self.label {
            Some(label) => vec![Tree::new(label)],
            None => Vec::new(),
        }
    }

    fn diff(&self, tree: &mut Tree) {
        match &self.label {
            Some(label) => tree.diff_children(std::slice::from_ref(label)),
            None => tree.diff_children(&[] as &[Element<'_, Message, Theme, Renderer>]),
        }
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
        let box_size = self.size;
        // The theme is not available in `layout()`, so derive the gap
        // between the box and the label from the box size to keep
        // layout deterministic across themes.
        let gap = box_size * (8.0 / DEFAULT_BOX_SIZE);

        let mut children = Vec::new();

        // The box occupies the leading region.
        let mut total_width = box_size;
        let mut total_height = box_size;

        if let Some(label) = &mut self.label {
            let label_limits = layout::Limits::new(
                Size::ZERO,
                Size::new(
                    (limits.max().width - box_size - gap).max(0.0),
                    limits.max().height,
                ),
            );
            let mut label_node =
                label
                    .as_widget_mut()
                    .layout(&mut tree.children[0], renderer, &label_limits);

            total_height = total_height.max(label_node.size().height);
            let label_y = (total_height - label_node.size().height) / 2.0;
            label_node = label_node.move_to(Point::new(box_size + gap, label_y));
            total_width += gap + label_node.size().width;
            children.push(label_node);
        }

        layout::Node::with_children(Size::new(total_width, total_height), children)
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
        let internal = tree.state.downcast_ref::<InternalState>();
        let bounds = layout.bounds();
        let box_size = self.size;

        let status = if !self.is_interactive() {
            Status::Disabled
        } else if internal.is_pressed {
            Status::Pressed
        } else if internal.is_hovered {
            Status::Hovered
        } else {
            Status::Active
        };

        let state = self.value.state();
        let checkbox_style = Catalog::style(theme, &self.class, state, status);

        // The box is vertically centered within the widget bounds.
        let box_y = bounds.y + (bounds.height - box_size) / 2.0;
        let box_bounds = Rectangle {
            x: bounds.x,
            y: box_y,
            width: box_size,
            height: box_size,
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: box_bounds,
                border: checkbox_style.border,
                ..renderer::Quad::default()
            },
            checkbox_style
                .box_background
                .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
        );

        // Draw the mark glyph, if any.
        if let Some((glyph, font)) = mark_glyph(state) {
            renderer.fill_text(
                Text {
                    content: glyph.to_string(),
                    bounds: Size::new(box_bounds.width, box_bounds.height),
                    size: Pixels(box_size),
                    line_height: text::LineHeight::Relative(1.0),
                    font,
                    align_x: alignment::Horizontal::Center.into(),
                    align_y: alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                },
                Point::new(
                    box_bounds.x + box_bounds.width / 2.0,
                    box_bounds.y + box_bounds.height / 2.0,
                ),
                checkbox_style.mark_color,
                box_bounds,
            );
        }

        // Draw the label child, if any.
        if let Some(label) = &self.label {
            let text_style = renderer::Style {
                text_color: checkbox_style.label_color,
            };
            if let Some(label_layout) = layout.children().next() {
                label.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    &text_style,
                    label_layout,
                    cursor,
                    viewport,
                );
            }
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
        // Forward events to the label child.
        if let Some(label) = &mut self.label
            && let Some(label_layout) = layout.children().next()
        {
            label.as_widget_mut().update(
                &mut tree.children[0],
                event,
                label_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        if !self.is_interactive() {
            return;
        }

        let internal = tree.state.downcast_mut::<InternalState>();
        let bounds = layout.bounds();
        let is_over = cursor.is_over(bounds);

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                internal.is_hovered = is_over;
                if !is_over {
                    internal.is_pressed = false;
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if is_over => {
                internal.is_pressed = true;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if internal.is_pressed
                    && is_over
                    && let Some(f) = &self.on_toggle
                {
                    let next = if self.cycle_indeterminate {
                        self.value.cycled()
                    } else {
                        self.value.toggled()
                    };
                    shell.publish(f(next));
                }
                internal.is_pressed = false;
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
        if let Some(label) = &mut self.label
            && let Some(label_layout) = layout.children().next()
        {
            label
                .as_widget_mut()
                .operate(&mut tree.children[0], label_layout, renderer, operation);
        }
    }
}

impl<'a, V, Message, Theme, Renderer> From<Checkbox<'a, V, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    V: Value + 'a,
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = iced::Font> + 'a,
{
    fn from(checkbox: Checkbox<'a, V, Message, Theme, Renderer>) -> Self {
        Element::new(checkbox)
    }
}
