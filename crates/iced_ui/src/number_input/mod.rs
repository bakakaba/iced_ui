//! A numeric input field with optional stepper buttons.
//!
//! [`NumberInput`] wraps iced's built-in [`text_input`] widget,
//! delegating all text editing, cursor management, and rendering to
//! iced. The consumer only provides and receives the numeric value `T`.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::number_input::NumberInput;
//!
//! let input = NumberInput::new(self.temperature)
//!     .on_change(Message::TemperatureChanged)
//!     .range(0.0..=100.0)
//!     .step(0.5);
//! ```

pub mod numeric;

pub use numeric::Numeric;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::text::{self, Renderer as _};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::widget::text_input::{self, TextInput};
use iced::{Element, Event, Length, Rectangle, Size};

use std::ops::RangeInclusive;

use crate::text_input::style::{self as input_style, Variant};
use crate::theme::Theme;

/// Internal message type for the wrapped text_input.
#[derive(Debug, Clone)]
enum InternalMsg {
    TextChanged(String),
    Submit,
}

/// Internal state stored in the widget tree.
struct State {
    /// The current text displayed in the input.
    text: String,
    /// Whether the current text is a valid in-range value.
    is_valid: bool,
}

/// A numeric input field with optional stepper (+/−) buttons.
///
/// Wraps iced's built-in `text_input` for text editing and rendering.
/// The consumer only holds the numeric value `T`, not a String.
pub struct NumberInput<'a, T, Message>
where
    T: Numeric,
{
    value: T,
    on_change: Option<Box<dyn Fn(T) -> Message + 'a>>,
    range: Option<RangeInclusive<T>>,
    step: T,
    show_stepper: bool,
    width: Length,
    precision: Option<usize>,
    variant: Variant,
    roundness: Option<crate::Roundness>,
}

impl<'a, T, Message> NumberInput<'a, T, Message>
where
    T: Numeric,
{
    /// Creates a new number input displaying the given value.
    pub fn new(value: T) -> Self {
        Self {
            value,
            on_change: None,
            range: None,
            step: T::zero(),
            show_stepper: true,
            width: Length::Fixed(120.0),
            precision: None,
            variant: Variant::default(),
            roundness: None,
        }
    }

    /// Sets the handler called when the numeric value changes.
    pub fn on_change(mut self, f: impl Fn(T) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Sets the valid range for the input value.
    pub fn range(mut self, range: RangeInclusive<T>) -> Self {
        self.range = Some(range);
        self
    }

    /// Sets the step size for stepper buttons and arrow keys.
    pub fn step(mut self, step: T) -> Self {
        self.step = step;
        self
    }

    /// Shows or hides the stepper (+/−) buttons.
    pub fn stepper(mut self, show: bool) -> Self {
        self.show_stepper = show;
        self
    }

    /// Sets the width of the input.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the number of decimal places to display (floats only).
    pub fn precision(mut self, digits: usize) -> Self {
        self.precision = Some(digits);
        self
    }

    /// Sets the visual variant (outlined or filled).
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides the corner roundness, bypassing the theme's default
    /// for this widget. Accepts a [`Roundness`](crate::Roundness)
    /// token: [`Roundness::sx`](crate::Roundness::sx) scales the
    /// theme's roundness base, [`Roundness::px`](crate::Roundness::px)
    /// sets an absolute radius.
    pub fn roundness(mut self, roundness: crate::Roundness) -> Self {
        self.roundness = Some(roundness);
        self
    }

    /// Formats the current value for display.
    fn format_value(&self) -> String {
        if let Some(precision) = self.precision {
            format!("{:.prec$}", self.value, prec = precision)
        } else {
            format!("{}", self.value)
        }
    }

    /// Formats an arbitrary value for display.
    fn format_t(&self, value: T) -> String {
        if let Some(precision) = self.precision {
            format!("{:.prec$}", value, prec = precision)
        } else {
            format!("{}", value)
        }
    }

    /// Clamps a value to the configured range (or returns it unchanged).
    fn clamp(&self, value: T) -> T {
        if let Some(range) = &self.range {
            value.clamp_range(range)
        } else {
            value
        }
    }

    /// Checks if a parsed value is within the configured range.
    fn in_range(&self, value: T) -> bool {
        if let Some(range) = &self.range {
            value >= *range.start() && value <= *range.end()
        } else {
            true
        }
    }

    /// Validates the entire string: every character must be valid for T.
    fn is_valid_text(text: &str) -> bool {
        if text.is_empty() {
            return true;
        }
        // Build up char-by-char to simulate sequential typing
        let mut so_far = String::new();
        for c in text.chars() {
            if !T::is_valid_char(c, &so_far) {
                return false;
            }
            so_far.push(c);
        }
        true
    }

    /// Steps the value up or down, clamped to range.
    fn step_value(&self, up: bool) -> T {
        let new_val = if up {
            self.value.add(self.step)
        } else {
            self.value.sub(self.step)
        };
        self.clamp(new_val)
    }
}

/// Builds the inner iced text_input widget using the given text.
/// This is a free function to avoid borrow issues with `self`.
fn build_inner_text_input<'a>(
    text: &'a str,
    variant: Variant,
) -> TextInput<'a, InternalMsg, Theme, iced::Renderer> {
    TextInput::new("", text)
        .width(Length::Fill)
        .padding([4, 0])
        .on_input(InternalMsg::TextChanged)
        .on_submit(InternalMsg::Submit)
        .style(move |theme: &Theme, status| {
            let our_status = match status {
                text_input::Status::Active => input_style::Status::Active,
                text_input::Status::Hovered => input_style::Status::Hovered,
                text_input::Status::Focused { .. } => input_style::Status::Focused,
                text_input::Status::Disabled => input_style::Status::Disabled,
            };
            let our_style = match variant {
                Variant::Outlined => input_style::outlined(theme, our_status),
                Variant::Filled => input_style::filled(theme, our_status),
            };
            // Inner text_input is transparent — the outer container draws
            // the background/border.
            text_input::Style {
                background: iced::Background::Color(iced::Color::TRANSPARENT),
                border: iced::Border::default(),
                icon: our_style.icon_color,
                placeholder: our_style.placeholder_color,
                value: our_style.value_color,
                selection: our_style.selection_color,
            }
        })
}

type Paragraph = <iced::Renderer as text::Renderer>::Paragraph;

impl<'a, T, Message> Widget<Message, Theme, iced::Renderer> for NumberInput<'a, T, Message>
where
    T: Numeric,
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            text: self.format_value(),
            is_valid: true,
        })
    }

    fn children(&self) -> Vec<Tree> {
        let formatted = self.format_value();
        let inner = build_inner_text_input(&formatted, self.variant);
        vec![Tree::new(
            &inner as &dyn Widget<InternalMsg, Theme, iced::Renderer>,
        )]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        // Check if the text_input is focused
        let is_focused = tree
            .children
            .first()
            .map(|child| {
                child
                    .state
                    .downcast_ref::<text_input::State<Paragraph>>()
                    .is_focused()
            })
            .unwrap_or(false);

        // If not focused, sync text with the current value
        if !is_focused {
            let formatted = self.format_value();
            if state.text != formatted {
                state.text = formatted;
                state.is_valid = true;
            }
        }

        // Ensure children vector has one entry
        if tree.children.is_empty() {
            let inner = build_inner_text_input(&state.text, self.variant);
            tree.children.push(Tree::new(
                &inner as &dyn Widget<InternalMsg, Theme, iced::Renderer>,
            ));
        }
        // Do NOT diff the child text_input — same as ComboBox pattern.
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
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_ref::<State>();
        let stepper_width: f32 = if self.show_stepper { 48.0 } else { 0.0 };
        let padding_h: f32 = 12.0;
        let padding_v: f32 = 8.0;

        // Build text_input and lay it out
        let text_snapshot = state.text.clone();
        let mut inner = build_inner_text_input(&text_snapshot, self.variant);

        let inner_limits = limits.width(self.width);
        let text_input_max_width =
            (inner_limits.max().width - stepper_width - padding_h * 2.0).max(0.0);

        let text_input_limits = layout::Limits::new(
            Size::ZERO,
            Size::new(text_input_max_width, inner_limits.max().height),
        )
        .width(Length::Fill);

        let mut text_node = inner.layout(&mut tree.children[0], renderer, &text_input_limits, None);

        let height = text_node.bounds().height + padding_v * 2.0;

        // Position the text_input child inside padding
        text_node = text_node.move_to(iced::Point::new(padding_h, padding_v));

        // Compute total width
        let total_width = match self.width {
            Length::Fixed(w) => w,
            Length::Fill | Length::FillPortion(_) => inner_limits.max().width,
            Length::Shrink => text_node.bounds().width + stepper_width + padding_h * 2.0,
        };

        layout::Node::with_children(Size::new(total_width, height), vec![text_node])
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // Detect focus state before processing the event
        let was_focused = tree.children[0]
            .state
            .downcast_ref::<text_input::State<Paragraph>>()
            .is_focused();

        // Handle stepper button clicks
        if self.show_stepper
            && let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && let Some(pos) = cursor.position_over(bounds)
        {
            let stepper_btn_w = 24.0;
            let minus_x = bounds.x + bounds.width - stepper_btn_w * 2.0;
            let plus_x = bounds.x + bounds.width - stepper_btn_w;

            if pos.x >= plus_x {
                // Plus button
                let new_val = self.step_value(true);
                let state = tree.state.downcast_mut::<State>();
                state.text = self.format_t(new_val);
                state.is_valid = true;
                if let Some(on_change) = &self.on_change {
                    shell.publish((on_change)(new_val));
                }
                shell.capture_event();
                shell.request_redraw();
                return;
            } else if pos.x >= minus_x {
                // Minus button
                let new_val = self.step_value(false);
                let state = tree.state.downcast_mut::<State>();
                state.text = self.format_t(new_val);
                state.is_valid = true;
                if let Some(on_change) = &self.on_change {
                    shell.publish((on_change)(new_val));
                }
                shell.capture_event();
                shell.request_redraw();
                return;
            }
        }

        // Clone the text to avoid borrow conflict: we need to create the
        // text_input from &str, but later mutate state.text based on messages.
        let text_snapshot = tree.state.downcast_ref::<State>().text.clone();

        // Forward event to the inner text_input using a local shell
        let mut local_messages: Vec<InternalMsg> = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let text_input_layout = layout.children().next().unwrap();

        let mut inner = build_inner_text_input(&text_snapshot, self.variant);
        inner.update(
            &mut tree.children[0],
            event,
            text_input_layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );

        // Forward shell state
        if local_shell.is_event_captured() {
            shell.capture_event();
        }
        shell.request_redraw_at(local_shell.redraw_request());
        shell.request_input_method(local_shell.input_method());

        // Now we can mutate state freely
        let state = tree.state.downcast_mut::<State>();

        // Process intercepted messages
        for msg in local_messages {
            match msg {
                InternalMsg::TextChanged(new_text) => {
                    // Character-level filtering
                    if !Self::is_valid_text(&new_text) {
                        // Reject: don't update state.text. Next frame, the
                        // text_input will receive the old text and revert.
                        shell.request_redraw();
                        continue;
                    }

                    // Accept the text
                    state.text = new_text;

                    // Try to parse and validate
                    if state.text.is_empty() {
                        state.is_valid = true; // empty is acceptable while typing
                    } else if let Ok(parsed) = state.text.parse::<T>() {
                        if self.in_range(parsed) {
                            state.is_valid = true;
                            if let Some(on_change) = &self.on_change {
                                shell.publish((on_change)(parsed));
                            }
                        } else {
                            state.is_valid = false;
                        }
                    } else {
                        // Intermediate state (e.g. "-", "1.")
                        state.is_valid = false;
                    }
                    shell.request_redraw();
                }
                InternalMsg::Submit => {
                    // On Enter: clamp and commit
                    if let Ok(parsed) = state.text.parse::<T>() {
                        let clamped = self.clamp(parsed);
                        state.text = self.format_t(clamped);
                        state.is_valid = true;
                        if let Some(on_change) = &self.on_change {
                            shell.publish((on_change)(clamped));
                        }
                    } else {
                        // Revert to current value
                        state.text = self.format_value();
                        state.is_valid = true;
                    }
                    shell.request_redraw();
                }
            }
        }

        // Detect blur (was focused, now isn't)
        let is_focused = tree.children[0]
            .state
            .downcast_ref::<text_input::State<Paragraph>>()
            .is_focused();

        if was_focused && !is_focused {
            let state = tree.state.downcast_mut::<State>();
            if let Ok(parsed) = state.text.parse::<T>() {
                let clamped = self.clamp(parsed);
                state.text = self.format_t(clamped);
                state.is_valid = true;
                if let Some(on_change) = &self.on_change {
                    shell.publish((on_change)(clamped));
                }
            } else {
                // Invalid text — revert
                state.text = self.format_value();
                state.is_valid = true;
            }
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        // Determine focus state
        let is_focused = tree.children[0]
            .state
            .downcast_ref::<text_input::State<Paragraph>>()
            .is_focused();

        let palette = theme.extended_palette();
        let roundness = match self.roundness {
            Some(r) => theme.radius(r),
            None => theme.radius(crate::Roundness::sx(1.0)),
        };

        // Compute outer container style
        let (bg, border_color, border_width) = if !state.is_valid {
            let bg = match self.variant {
                Variant::Outlined => iced::Color::TRANSPARENT,
                Variant::Filled => palette.background.neutral.color,
            };
            (bg, palette.danger.base.color, 1.5)
        } else if is_focused {
            let bg = match self.variant {
                Variant::Outlined => iced::Color::TRANSPARENT,
                Variant::Filled => palette.background.neutral.color,
            };
            (bg, palette.primary.base.color, 1.5)
        } else {
            match self.variant {
                Variant::Outlined => (
                    iced::Color::TRANSPARENT,
                    palette.background.strong.color,
                    1.0,
                ),
                Variant::Filled => (
                    palette.background.neutral.color,
                    iced::Color::TRANSPARENT,
                    0.0,
                ),
            }
        };

        // Draw outer background + border
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    color: border_color,
                    width: border_width,
                    radius: roundness.into(),
                },
                shadow: iced::Shadow::default(),
                ..renderer::Quad::default()
            },
            iced::Background::Color(bg),
        );

        // Draw the inner text_input
        let text_input_layout = layout.children().next().unwrap();
        let inner = build_inner_text_input(&state.text, self.variant);
        inner.draw(
            &tree.children[0],
            renderer,
            theme,
            text_input_layout,
            cursor,
            None,
            viewport,
        );

        // Draw stepper buttons
        if self.show_stepper {
            let stepper_btn_w = 24.0;
            let button_height = bounds.height;
            let stepper_color = palette.background.strong.color;
            let font_size = iced::Pixels(14.0);

            // Minus button
            let minus_bounds = Rectangle {
                x: bounds.x + bounds.width - stepper_btn_w * 2.0,
                y: bounds.y,
                width: stepper_btn_w,
                height: button_height,
            };
            renderer.fill_text(
                iced::advanced::Text {
                    content: "\u{2212}".to_string(),
                    bounds: Size::new(stepper_btn_w, button_height),
                    size: font_size,
                    line_height: text::LineHeight::Relative(1.0),
                    font: renderer.default_font(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                },
                iced::Point::new(
                    minus_bounds.x + minus_bounds.width / 2.0,
                    minus_bounds.y + minus_bounds.height / 2.0,
                ),
                stepper_color,
                minus_bounds,
            );

            // Plus button
            let plus_bounds = Rectangle {
                x: bounds.x + bounds.width - stepper_btn_w,
                y: bounds.y,
                width: stepper_btn_w,
                height: button_height,
            };
            renderer.fill_text(
                iced::advanced::Text {
                    content: "+".to_string(),
                    bounds: Size::new(stepper_btn_w, button_height),
                    size: font_size,
                    line_height: text::LineHeight::Relative(1.0),
                    font: renderer.default_font(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                },
                iced::Point::new(
                    plus_bounds.x + plus_bounds.width / 2.0,
                    plus_bounds.y + plus_bounds.height / 2.0,
                ),
                stepper_color,
                plus_bounds,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();

        if let Some(pos) = cursor.position_over(bounds) {
            // Pointer cursor over stepper buttons
            if self.show_stepper {
                let stepper_btn_w = 24.0;
                let minus_x = bounds.x + bounds.width - stepper_btn_w * 2.0;
                if pos.x >= minus_x {
                    return mouse::Interaction::Pointer;
                }
            }

            // Delegate to text_input for the text area
            let text_input_layout = layout.children().next().unwrap();
            let state = tree.state.downcast_ref::<State>();
            let inner = build_inner_text_input(&state.text, self.variant);
            return inner.mouse_interaction(
                &tree.children[0],
                text_input_layout,
                cursor,
                viewport,
                renderer,
            );
        }

        mouse::Interaction::default()
    }
}

impl<'a, T, Message> From<NumberInput<'a, T, Message>> for Element<'a, Message, Theme>
where
    T: Numeric,
    Message: Clone + 'a,
{
    fn from(input: NumberInput<'a, T, Message>) -> Self {
        Element::new(input)
    }
}
