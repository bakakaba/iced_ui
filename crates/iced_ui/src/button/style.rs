//! Styling primitives for the [`Button`](super::Button) widget.

use iced::{Background, Border, Color, Shadow};

use crate::{Roundness, Theme};

/// The visual variant of a button.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    /// Filled background with contrasting text. This is the default.
    #[default]
    Solid,
    /// Border outline with no fill; text uses the button's color token.
    Outline,
    /// Transparent at rest; gains a subtle background on hover.
    Ghost,
}

/// The size of a button.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonSize {
    /// Small: 24px height.
    Sm,
    /// Medium: 32px height. This is the default.
    #[default]
    Md,
    /// Large: 40px height.
    Lg,
}

impl ButtonSize {
    /// Returns the button height in logical pixels.
    pub fn height(self) -> f32 {
        match self {
            Self::Sm => 24.0,
            Self::Md => 32.0,
            Self::Lg => 40.0,
        }
    }

    /// Returns the horizontal padding in logical pixels.
    pub fn h_padding(self) -> f32 {
        match self {
            Self::Sm => 8.0,
            Self::Md => 12.0,
            Self::Lg => 16.0,
        }
    }

    /// Returns the font size as a fraction of the given base text size.
    ///
    /// Pass the theme's base text size (e.g. `theme.text_size()`).
    pub fn font_size(self, base: f32) -> f32 {
        match self {
            Self::Sm => base * 0.75,
            Self::Md => base * 0.8125,
            Self::Lg => base * 0.875,
        }
    }
}

/// The color token applied to a button's text and hover derivation.
///
/// Each variant resolves to a concrete [`Color`] from the active
/// theme at draw time. The resolved color drives:
///
/// - **Solid**: the background fill (text is the readable contrast).
/// - **Outline**: the text and border color.
/// - **Ghost**: the text color.
///
/// Hover and press overlays are derived from the resolved color.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ButtonColor {
    /// Primary color from the theme.
    #[default]
    Primary,
    /// Secondary color from the theme.
    Secondary,
    /// Success color from the theme.
    Success,
    /// Warning color from the theme.
    Warning,
    /// Danger color from the theme.
    Danger,
    /// Foreground/text color from the theme.
    Foreground,
    /// Informational color from the theme.
    Information,
    /// A specific custom color.
    Custom(Color),
}

/// The interaction status of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Normal state.
    Active,
    /// Pointer hovering.
    Hovered,
    /// Being pressed.
    Pressed,
    /// Logically focused (e.g. keyboard navigation, active menu
    /// trigger). Shown when the button is not hovered or pressed.
    Focused,
    /// Button is disabled.
    Disabled,
}

/// The visual style of a [`Button`](super::Button).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background fill.
    pub background: Option<Background>,
    /// Label/icon color.
    pub text_color: Color,
    /// Border.
    pub border: Border,
    /// Shadow.
    pub shadow: Shadow,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            text_color: Color::WHITE,
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }
}

/// A function that returns a [`Style`] for a given theme, variant,
/// size, color token, and interaction status.
pub type StyleFn<'a, Theme> =
    Box<dyn Fn(&Theme, Variant, ButtonSize, ButtonColor, Status) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Button`](super::Button).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class, variant, size, color
    /// token, and status.
    fn style(
        &self,
        class: &Self::Class<'_>,
        variant: Variant,
        size: ButtonSize,
        color: ButtonColor,
        status: Status,
    ) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(
        &self,
        class: &Self::Class<'_>,
        variant: Variant,
        size: ButtonSize,
        color: ButtonColor,
        status: Status,
    ) -> Style {
        class(self, variant, size, color, status)
    }
}

/// The default button style.
///
/// Resolves the [`ButtonColor`] token against the theme to obtain
/// a concrete color, then applies it according to the [`Variant`]:
///
/// - **Solid**: resolved color as background, readable contrast text.
/// - **Outline**: transparent background, resolved color for text and
///   border.
/// - **Ghost**: fully transparent, resolved color for text.
///
/// All variants use the theme's roundness token for border radius.
pub fn default(
    theme: &Theme,
    variant: Variant,
    _size: ButtonSize,
    color: ButtonColor,
    status: Status,
) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(2.0));

    // Resolve the color token to a concrete color and its readable
    // contrast (for use as text on a solid background).
    let (resolved, on_resolved) = match color {
        ButtonColor::Primary => (palette.primary.base.color, palette.primary.base.text),
        ButtonColor::Secondary => (palette.secondary.base.color, palette.secondary.base.text),
        ButtonColor::Success => (palette.success.base.color, palette.success.base.text),
        ButtonColor::Warning => (palette.warning.base.color, palette.warning.base.text),
        ButtonColor::Danger => (palette.danger.base.color, palette.danger.base.text),
        ButtonColor::Foreground => (palette.background.base.text, palette.background.base.color),
        ButtonColor::Information => (theme.information.base.color, theme.information.base.text),
        ButtonColor::Custom(c) => (c, palette.background.base.color),
    };

    let base = match variant {
        Variant::Solid => Style {
            background: Some(Background::Color(resolved)),
            text_color: on_resolved,
            border: Border {
                radius: radius.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
        },
        Variant::Outline => Style {
            background: None,
            text_color: resolved,
            border: Border {
                radius: radius.into(),
                width: 1.0,
                color: resolved,
            },
            shadow: Shadow::default(),
        },
        Variant::Ghost => Style {
            background: None,
            text_color: resolved,
            border: Border {
                radius: radius.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
        },
    };

    match status {
        Status::Active => base,
        Status::Hovered | Status::Focused => {
            let alpha = match variant {
                Variant::Solid => 0.08,
                Variant::Outline | Variant::Ghost => 0.15,
            };
            let overlay = Color {
                a: alpha,
                ..resolved
            };
            Style {
                background: Some(Background::Color(match base.background {
                    Some(Background::Color(c)) => blend_over(c, overlay),
                    _ => overlay,
                })),
                ..base
            }
        }
        Status::Pressed => {
            let alpha = match variant {
                Variant::Solid => 0.12,
                Variant::Outline | Variant::Ghost => 0.25,
            };
            let overlay = Color {
                a: alpha,
                ..resolved
            };
            Style {
                background: Some(Background::Color(match base.background {
                    Some(Background::Color(c)) => blend_over(c, overlay),
                    _ => overlay,
                })),
                ..base
            }
        }
        Status::Disabled => {
            let disabled_text = Color {
                a: 0.50,
                ..palette.background.base.text
            };
            let disabled_surface = Color {
                a: 0.15,
                ..palette.background.base.text
            };
            match variant {
                Variant::Solid => Style {
                    background: Some(Background::Color(disabled_surface)),
                    text_color: disabled_text,
                    border: Border {
                        radius: radius.into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                },
                Variant::Outline => Style {
                    background: None,
                    text_color: disabled_text,
                    border: Border {
                        radius: radius.into(),
                        width: 1.0,
                        color: Color {
                            a: 0.30,
                            ..palette.background.base.text
                        },
                    },
                    shadow: Shadow::default(),
                },
                Variant::Ghost => Style {
                    background: None,
                    text_color: disabled_text,
                    border: Border {
                        radius: radius.into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                },
            }
        }
    }
}

/// Simple alpha-over compositing.
fn blend_over(dst: Color, src: Color) -> Color {
    let a = src.a;
    Color {
        r: dst.r * (1.0 - a) + src.r * a,
        g: dst.g * (1.0 - a) + src.g * a,
        b: dst.b * (1.0 - a) + src.b * a,
        a: dst.a + a * (1.0 - dst.a),
    }
}
