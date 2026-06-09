//! Styling primitives for the [`Chip`](super::Chip) widget.

use iced::{Background, Border, Color, Shadow};

use crate::{Roundness, Theme};

/// The kind of chip.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Guides the user toward a specific action (e.g. "Add to
    /// calendar").
    #[default]
    Assist,
    /// Toggles a filter on or off in a group of options.
    Filter,
    /// Represents a discrete piece of information (e.g. contact name).
    Input,
    /// Offers a contextual suggestion (e.g. quick reply).
    Suggestion,
}

/// The interaction status of a chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Normal state.
    Active,
    /// Pointer hovering.
    Hovered,
    /// Being pressed.
    Pressed,
    /// Chip is disabled.
    Disabled,
}

/// The visual style of a [`Chip`](super::Chip).
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

/// A function that returns a [`Style`] for a given theme, kind,
/// selected state, and interaction status.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Kind, bool, Status) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Chip`](super::Chip).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class, kind, selected state,
    /// and status.
    fn style(&self, class: &Self::Class<'_>, kind: Kind, selected: bool, status: Status) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, kind: Kind, selected: bool, status: Status) -> Style {
        class(self, kind, selected, status)
    }
}

/// The default chip style.
pub fn default(theme: &Theme, kind: Kind, selected: bool, status: Status) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(1.0));

    let base = if selected {
        Style {
            background: Some(Background::Color(palette.primary.weak.color)),
            text_color: palette.primary.weak.text,
            border: Border {
                radius: radius.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
        }
    } else {
        match kind {
            Kind::Assist | Kind::Suggestion => Style {
                background: None,
                text_color: palette.background.base.text,
                border: Border {
                    radius: radius.into(),
                    width: 1.0,
                    color: palette.background.strong.color,
                },
                shadow: Shadow::default(),
            },
            Kind::Filter => Style {
                background: None,
                text_color: palette.background.base.text,
                border: Border {
                    radius: radius.into(),
                    width: 1.0,
                    color: palette.background.strong.color,
                },
                shadow: Shadow::default(),
            },
            Kind::Input => Style {
                background: None,
                text_color: palette.background.base.text,
                border: Border {
                    radius: radius.into(),
                    width: 1.0,
                    color: palette.background.strong.color,
                },
                shadow: Shadow::default(),
            },
        }
    };

    match status {
        Status::Active => base,
        Status::Hovered => {
            let hover = Color {
                a: 0.08,
                ..base.text_color
            };
            Style {
                background: Some(Background::Color(match base.background {
                    Some(Background::Color(c)) => blend_over(c, hover),
                    _ => hover,
                })),
                ..base
            }
        }
        Status::Pressed => {
            let press = Color {
                a: 0.12,
                ..base.text_color
            };
            Style {
                background: Some(Background::Color(match base.background {
                    Some(Background::Color(c)) => blend_over(c, press),
                    _ => press,
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
            text_color: Color {
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
