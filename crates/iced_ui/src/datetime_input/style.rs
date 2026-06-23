//! Styling for the popup panel shared by [`DateInput`],
//! [`TimeInput`], and [`DateTimeInput`].
//!
//! The text field part of those widgets reuses the
//! [`text_input`](crate::text_input::style) styling (like
//! [`NumberInput`](crate::NumberInput) does); this module only styles
//! the floating picker panel.
//!
//! [`DateInput`]: crate::DateInput
//! [`TimeInput`]: crate::TimeInput
//! [`DateTimeInput`]: crate::DateTimeInput

use iced::{Background, Border, Color};

use crate::Theme;
use crate::theme::scale::Roundness;

/// The appearance of the picker popup panel.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// Background of the popup panel.
    pub popup_background: Background,
    /// Border around the popup panel.
    pub popup_border: Border,
    /// Color of the month/year title and the navigation chevrons.
    pub title_color: Color,
    /// Color of the weekday header labels and section labels.
    pub label_color: Color,
    /// Text color of a regular, selectable cell.
    pub cell_text_color: Color,
    /// Text color of muted cells (days outside the view month or
    /// outside the allowed range).
    pub cell_muted_color: Color,
    /// Background of the cell currently under the cursor.
    pub cell_hover_background: Background,
    /// Background of the selected cell.
    pub selected_background: Background,
    /// Text color of the selected cell.
    pub selected_text_color: Color,
    /// Border color of the cell marking today's date.
    pub today_border_color: Color,
    /// Color of the hour/minute column scrollbar thumbs.
    pub scrollbar_thumb: Color,
    /// Corner radius of cell highlights.
    pub cell_radius: f32,
}

/// A function that resolves the popup [`Style`] from a theme.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

/// The default popup style derived from the theme palette.
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(1.0));

    Style {
        popup_background: Background::Color(palette.background.base.color),
        popup_border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: radius.into(),
        },
        title_color: palette.background.base.text,
        label_color: palette.background.strong.color,
        cell_text_color: palette.background.base.text,
        cell_muted_color: palette.background.strong.color,
        cell_hover_background: Background::Color(palette.background.weak.color),
        selected_background: Background::Color(palette.primary.base.color),
        selected_text_color: palette.primary.base.text,
        today_border_color: palette.primary.base.color,
        scrollbar_thumb: palette.background.strong.color,
        cell_radius: radius * 0.5,
    }
}
