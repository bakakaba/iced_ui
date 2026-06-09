//! A themed text input with optional leading and trailing elements.
//!
//! Wraps iced's built-in `text_input` widget within a styled container
//! and provides leading/trailing `Element` slots for icons, buttons,
//! etc.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::text_input::TextInput;
//!
//! let input = TextInput::new("Search...", &self.query)
//!     .on_input(Message::QueryChanged);
//! ```

pub mod style;

pub use style::{Catalog, Status, Style, StyleFn, Variant, filled, outlined};

use iced::widget::{container, row, text_input};
use iced::{Element, Length};

use crate::Theme;

/// A themed text input with optional leading/trailing elements.
///
/// This is a builder that produces an [`Element`] via the [`Into`]
/// implementation. It composes iced's built-in `text_input` inside a
/// styled container with optional leading/trailing decoration slots.
pub struct TextInput<'a, Message> {
    placeholder: &'a str,
    value: &'a str,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    is_secure: bool,
    width: Length,
    leading: Option<Element<'a, Message, Theme>>,
    trailing: Option<Element<'a, Message, Theme>>,
    variant: Variant,
}

impl<'a, Message: Clone + 'a> TextInput<'a, Message> {
    /// Creates a new text input with the given placeholder and current
    /// value.
    pub fn new(placeholder: &'a str, value: &'a str) -> Self {
        Self {
            placeholder,
            value,
            on_input: None,
            on_submit: None,
            is_secure: false,
            width: Length::Fill,
            leading: None,
            trailing: None,
            variant: Variant::default(),
        }
    }

    /// Sets the handler called on every text change. Without this, the
    /// input is disabled (read-only).
    pub fn on_input(mut self, f: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(f));
        self
    }

    /// Sets the message produced when Enter is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Toggles secure (password) mode — characters are replaced with
    /// dots.
    pub fn secure(mut self, is_secure: bool) -> Self {
        self.is_secure = is_secure;
        self
    }

    /// Sets the width of the text input.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets a leading element (rendered before the text field).
    pub fn leading(mut self, element: impl Into<Element<'a, Message, Theme>>) -> Self {
        self.leading = Some(element.into());
        self
    }

    /// Sets a trailing element (rendered after the text field).
    pub fn trailing(mut self, element: impl Into<Element<'a, Message, Theme>>) -> Self {
        self.trailing = Some(element.into());
        self
    }

    /// Sets the visual variant (outlined or filled).
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }
}

impl<'a, Message: Clone + 'a> From<TextInput<'a, Message>> for Element<'a, Message, Theme> {
    fn from(input: TextInput<'a, Message>) -> Self {
        let variant = input.variant;

        // Build the inner iced text_input
        let mut inner = text_input(input.placeholder, input.value)
            .width(Length::Fill)
            .padding([4, 0])
            .style(move |theme: &Theme, status| {
                // Map iced's Status to our Status for color lookup
                let our_status = match status {
                    text_input::Status::Active => Status::Active,
                    text_input::Status::Hovered => Status::Hovered,
                    text_input::Status::Focused { .. } => Status::Focused,
                    text_input::Status::Disabled => Status::Disabled,
                };
                let our_style = match variant {
                    Variant::Outlined => outlined(theme, our_status),
                    Variant::Filled => filled(theme, our_status),
                };
                // Inner text_input is transparent — the container draws the
                // background/border.
                iced::widget::text_input::Style {
                    background: iced::Background::Color(iced::Color::TRANSPARENT),
                    border: iced::Border::default(),
                    icon: our_style.icon_color,
                    placeholder: our_style.placeholder_color,
                    value: our_style.value_color,
                    selection: our_style.selection_color,
                }
            });

        if let Some(on_input) = input.on_input {
            inner = inner.on_input(on_input);
        }
        if let Some(msg) = input.on_submit {
            inner = inner.on_submit(msg);
        }
        if input.is_secure {
            inner = inner.secure(true);
        }

        // Build the row: [leading?] [text_input] [trailing?]
        let mut content = row![].spacing(8).align_y(iced::Alignment::Center);

        if let Some(leading) = input.leading {
            content = content.push(leading);
        }

        content = content.push(inner);

        if let Some(trailing) = input.trailing {
            content = content.push(trailing);
        }

        // Wrap in a styled container
        container(content)
            .width(input.width)
            .padding([8, 12])
            .style(move |theme: &Theme| {
                let style = match variant {
                    Variant::Outlined => outlined(theme, Status::Active),
                    Variant::Filled => filled(theme, Status::Active),
                };
                iced::widget::container::Style {
                    background: Some(style.background),
                    border: style.border,
                    ..Default::default()
                }
            })
            .into()
    }
}
