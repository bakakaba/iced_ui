//! Styling primitives for the [`Snackbar`](super::Snackbar) widget.

use iced::{Background, Border, Color, Shadow};

use crate::{Elevation, Roundness, ShadowDir, Theme};

use super::Severity;

/// The visual style of a [`Snackbar`](super::Snackbar).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background of the snackbar bar.
    pub background: Background,
    /// Message text color.
    pub text_color: Color,
    /// Action button text color.
    pub action_color: Color,
    /// Border of the snackbar bar.
    pub border: Border,
    /// Shadow (elevation) of the snackbar bar.
    pub shadow: Shadow,
    /// Optional accent color for the severity left border and icon.
    /// When `None`, no severity indicator is drawn.
    pub severity_color: Option<Color>,
}

/// A function that returns a [`Style`] for a given theme and severity.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Severity) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Snackbar`](super::Snackbar).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class and severity.
    fn style(&self, class: &Self::Class<'_>, severity: Severity) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, severity: Severity) -> Style {
        class(self, severity)
    }
}

/// The default snackbar style.
pub fn default(theme: &Theme, severity: Severity) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(1.0));

    let severity_color = match severity {
        Severity::Neutral => None,
        Severity::Information => Some(theme.information.base.color),
        Severity::Success => Some(palette.success.base.color),
        Severity::Warning => Some(palette.warning.base.color),
        Severity::Error => Some(palette.danger.base.color),
    };

    Style {
        background: Background::Color(Color::from_rgb(
            0.196, 0.196, 0.196, // ~#323232
        )),
        text_color: Color::WHITE,
        action_color: palette.primary.base.color,
        border: Border {
            radius: radius.into(),
            ..Border::default()
        },
        shadow: theme.shadow(Elevation::sx(1.0), ShadowDir::Down),
        severity_color,
    }
}
