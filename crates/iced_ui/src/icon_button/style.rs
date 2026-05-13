//! Styling primitives for the [`IconButton`](super::IconButton)
//! widget.

use iced::{Background, Border, Color, Shadow};

use crate::{Roundness, Theme};

/// The visual variant of an icon button.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    /// No fill, no border. Icon color is primary on the surface.
    #[default]
    Standard,
    /// Primary-colored filled background.
    Filled,
    /// Secondary/tonal filled background.
    FilledTonal,
    /// Surface with a visible border.
    Outlined,
}

/// The interaction status of an icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Normal state.
    Active,
    /// Pointer hovering over the button.
    Hovered,
    /// Button is currently pressed.
    Pressed,
    /// Button is disabled.
    Disabled,
}

/// The visual style of an [`IconButton`](super::IconButton).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background fill.
    pub background: Option<Background>,
    /// Icon/text color.
    pub icon_color: Color,
    /// Border.
    pub border: Border,
    /// Shadow.
    pub shadow: Shadow,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            icon_color: Color::WHITE,
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }
}

/// A function that returns a [`Style`] for a given theme, variant,
/// status, and toggle state.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Variant, Status, bool) -> Style + 'a>;

/// Catalog of theme-driven styles for an
/// [`IconButton`](super::IconButton).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class, variant, status, and
    /// toggle state.
    fn style(
        &self,
        class: &Self::Class<'_>,
        variant: Variant,
        status: Status,
        toggled: bool,
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
        status: Status,
        toggled: bool,
    ) -> Style {
        class(self, variant, status, toggled)
    }
}

/// The default icon button styling.
pub fn default(theme: &Theme, variant: Variant, status: Status, toggled: bool) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(5.0));

    let base = match variant {
        Variant::Standard => {
            if toggled {
                Style {
                    background: None,
                    icon_color: palette.primary.base.color,
                    border: Border {
                        radius: radius.into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                }
            } else {
                Style {
                    background: None,
                    icon_color: palette.background.base.text,
                    border: Border {
                        radius: radius.into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                }
            }
        }
        Variant::Filled => {
            if toggled {
                Style {
                    background: Some(Background::Color(palette.primary.base.color)),
                    icon_color: palette.primary.base.text,
                    border: Border {
                        radius: radius.into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                }
            } else {
                Style {
                    background: Some(Background::Color(palette.background.strong.color)),
                    icon_color: palette.primary.base.color,
                    border: Border {
                        radius: radius.into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                }
            }
        }
        Variant::FilledTonal => {
            if toggled {
                Style {
                    background: Some(Background::Color(palette.primary.weak.color)),
                    icon_color: palette.primary.weak.text,
                    border: Border {
                        radius: radius.into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                }
            } else {
                Style {
                    background: Some(Background::Color(palette.background.weak.color)),
                    icon_color: palette.background.weak.text,
                    border: Border {
                        radius: radius.into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                }
            }
        }
        Variant::Outlined => {
            if toggled {
                Style {
                    background: Some(Background::Color(palette.primary.weak.color)),
                    icon_color: palette.primary.weak.text,
                    border: Border {
                        radius: radius.into(),
                        width: 1.0,
                        color: palette.primary.weak.color,
                    },
                    shadow: Shadow::default(),
                }
            } else {
                Style {
                    background: None,
                    icon_color: palette.background.base.text,
                    border: Border {
                        radius: radius.into(),
                        width: 1.0,
                        color: palette.background.strong.color,
                    },
                    shadow: Shadow::default(),
                }
            }
        }
    };

    match status {
        Status::Active => base,
        Status::Hovered => {
            let hover_color = Color {
                a: 0.08,
                ..base.icon_color
            };
            Style {
                background: Some(Background::Color(match base.background {
                    Some(Background::Color(c)) => blend_over(c, hover_color),
                    _ => hover_color,
                })),
                ..base
            }
        }
        Status::Pressed => {
            let press_color = Color {
                a: 0.12,
                ..base.icon_color
            };
            Style {
                background: Some(Background::Color(match base.background {
                    Some(Background::Color(c)) => blend_over(c, press_color),
                    _ => press_color,
                })),
                ..base
            }
        }
        Status::Disabled => Style {
            background: base.background.map(|_| {
                Background::Color(Color {
                    a: 0.12,
                    ..palette.background.strong.color
                })
            }),
            icon_color: Color {
                a: 0.38,
                ..palette.background.base.text
            },
            border: Border {
                color: Color {
                    a: 0.12,
                    ..base.border.color
                },
                ..base.border
            },
            shadow: Shadow::default(),
        },
    }
}

/// Simple alpha-over compositing (src over dst, both premultiplied
/// assumption ignored for simplicity).
fn blend_over(dst: Color, src: Color) -> Color {
    let a = src.a;
    Color {
        r: dst.r * (1.0 - a) + src.r * a,
        g: dst.g * (1.0 - a) + src.g * a,
        b: dst.b * (1.0 - a) + src.b * a,
        a: dst.a + a * (1.0 - dst.a),
    }
}
