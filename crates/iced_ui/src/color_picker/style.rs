//! Styling for the [`ColorPicker`](super::ColorPicker) widget.

use iced::{Background, Border, Color};

use crate::Theme;
use crate::theme::scale::Roundness;

/// The appearance of a [`ColorPicker`](super::ColorPicker).
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// Border around the trigger swatch.
    pub trigger_border: Border,
    /// Background of the popup panel.
    pub popup_background: Option<Background>,
    /// Border around the popup panel.
    pub popup_border: Border,
    /// Border around the SV area and slider bars.
    pub bar_border: Border,
    /// Color of the drag handle / crosshair (inner).
    pub handle_color: Color,
    /// Color of the drag handle / crosshair outline.
    pub handle_border: Color,
    /// Color of the color code text.
    pub text_color: Color,
}

/// Interaction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The widget is idle (popup closed).
    Idle,
    /// The popup is open.
    Open,
}

/// A style function for the color picker.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

/// The styling catalog for [`ColorPicker`](super::ColorPicker).
pub trait Catalog {
    /// The style class type.
    type Class<'a>;

    /// Returns the default style class.
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves the style for the given class and status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style for the color picker.
pub fn default(theme: &Theme, _status: Status) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(1.0));
    Style {
        trigger_border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: radius.into(),
        },
        popup_background: Some(Background::Color(palette.background.base.color)),
        popup_border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: radius.into(),
        },
        bar_border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: (radius * 0.5).into(),
        },
        handle_color: Color::WHITE,
        handle_border: Color::BLACK,
        text_color: palette.background.base.text,
    }
}
