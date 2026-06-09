//! Styling primitives for the [`TextInput`](super::TextInput) widget.

use iced::{Background, Border, Color};

use crate::Theme;

/// The visual variant of a text input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
    /// A text input with a visible border (default).
    #[default]
    Outlined,
    /// A text input with a tinted background and no border.
    Filled,
}

/// The interaction status of a text input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The text input is idle.
    Active,
    /// The cursor is over the text input.
    Hovered,
    /// The text input has keyboard focus.
    Focused,
    /// The text input is disabled (no `on_input` handler).
    Disabled,
}

/// The visual style of a [`TextInput`](super::TextInput).
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// Background fill of the input container.
    pub background: Background,
    /// Border of the input container.
    pub border: Border,
    /// Color used for leading/trailing icon elements.
    pub icon_color: Color,
    /// Color of the placeholder text.
    pub placeholder_color: Color,
    /// Color of the input value text.
    pub value_color: Color,
    /// Color of the text selection highlight.
    pub selection_color: Color,
}

/// A function that returns a [`Style`] for a given theme and status.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`TextInput`](super::TextInput).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class and status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(outlined)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default outlined text input style.
pub fn outlined(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let roundness = theme.radius(crate::Roundness::sx(1.0));

    let (border_color, bg) = match status {
        Status::Active => (palette.background.strong.color, Color::TRANSPARENT),
        Status::Hovered => (palette.background.strongest.color, Color::TRANSPARENT),
        Status::Focused => (palette.primary.base.color, Color::TRANSPARENT),
        Status::Disabled => (
            palette.background.weak.color,
            palette.background.weakest.color,
        ),
    };

    Style {
        background: Background::Color(bg),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: roundness.into(),
        },
        icon_color: palette.background.strongest.color,
        placeholder_color: palette.background.strong.color,
        value_color: palette.background.base.text,
        selection_color: palette.primary.weak.color,
    }
}

/// A filled text input style using the background neutral color.
pub fn filled(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let roundness = theme.radius(crate::Roundness::sx(1.0));

    let (border_color, bg) = match status {
        Status::Active => (Color::TRANSPARENT, palette.background.neutral.color),
        Status::Hovered => (Color::TRANSPARENT, palette.background.strong.color),
        Status::Focused => (palette.primary.base.color, palette.background.neutral.color),
        Status::Disabled => (Color::TRANSPARENT, palette.background.weak.color),
    };

    Style {
        background: Background::Color(bg),
        border: Border {
            color: border_color,
            width: if matches!(status, Status::Focused) {
                1.0
            } else {
                0.0
            },
            radius: roundness.into(),
        },
        icon_color: palette.background.strongest.color,
        placeholder_color: palette.background.strong.color,
        value_color: palette.background.base.text,
        selection_color: palette.primary.weak.color,
    }
}
