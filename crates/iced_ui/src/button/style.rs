//! Styling primitives for the [`Button`](super::Button) widget.

use iced::{Background, Border, Color, Shadow};

use crate::{Roundness, Theme};

/// The visual variant of a button.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    /// Filled background with contrasting text. This is the default.
    #[default]
    Solid,
    /// Border outline with no fill; text uses primary color.
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

    /// Returns the font size in logical pixels.
    pub fn font_size(self) -> f32 {
        match self {
            Self::Sm => 12.0,
            Self::Md => 13.0,
            Self::Lg => 14.0,
        }
    }
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
/// size, and interaction status.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Variant, ButtonSize, Status) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Button`](super::Button).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class, variant, size, and
    /// status.
    fn style(
        &self,
        class: &Self::Class<'_>,
        variant: Variant,
        size: ButtonSize,
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
        status: Status,
    ) -> Style {
        class(self, variant, size, status)
    }
}

/// The default button style.
///
/// - **Solid**: primary strong background, on-primary text.
/// - **Outline**: transparent, 1px primary border, primary text.
/// - **Ghost**: fully transparent, primary text.
///
/// All variants use the theme's roundness token for border radius.
pub fn default(theme: &Theme, variant: Variant, _size: ButtonSize, status: Status) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(2.0));

    let base = match variant {
        Variant::Solid => Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            text_color: palette.primary.strong.text,
            border: Border {
                radius: radius.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
        },
        Variant::Outline => Style {
            background: None,
            text_color: palette.primary.base.color,
            border: Border {
                radius: radius.into(),
                width: 1.0,
                color: palette.primary.base.color,
            },
            shadow: Shadow::default(),
        },
        Variant::Ghost => Style {
            background: None,
            text_color: palette.primary.base.color,
            border: Border {
                radius: radius.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
        },
    };

    match status {
        Status::Active => base,
        Status::Hovered => {
            let alpha = match variant {
                Variant::Solid => 0.08,
                Variant::Outline | Variant::Ghost => 0.15,
            };
            let overlay = Color {
                a: alpha,
                ..base.text_color
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
                ..base.text_color
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
