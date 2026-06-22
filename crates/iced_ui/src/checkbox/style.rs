//! Styling primitives for the [`Checkbox`](super::Checkbox) widget.

use iced::{Background, Border, Color};

use crate::{Roundness, Theme};

use super::State;

/// The interaction status of a checkbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Normal state.
    Active,
    /// Pointer hovering.
    Hovered,
    /// Being pressed.
    Pressed,
    /// Checkbox is disabled.
    Disabled,
}

/// The visual style of a [`Checkbox`](super::Checkbox).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Fill of the checkbox box. `None` renders a transparent box
    /// (typically used for the unchecked state).
    pub box_background: Option<Background>,
    /// Border of the box.
    pub border: Border,
    /// Color of the check / dash mark drawn inside the box.
    pub mark_color: Color,
    /// Color applied to the label element.
    pub label_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            box_background: None,
            border: Border::default(),
            mark_color: Color::WHITE,
            label_color: Color::BLACK,
        }
    }
}

/// A function that returns a [`Style`] for a given theme, logical
/// [`State`], and interaction [`Status`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, State, Status) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Checkbox`](super::Checkbox).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class, state, and status.
    fn style(&self, class: &Self::Class<'_>, state: State, status: Status) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, state: State, status: Status) -> Style {
        class(self, state, status)
    }
}

/// The default checkbox style.
pub fn default(theme: &Theme, state: State, status: Status) -> Style {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(0.25));

    // A box that has a visible mark (checked or indeterminate) is
    // filled with the primary color; an unchecked box is an outline.
    let base = if state.is_marked() {
        Style {
            box_background: Some(Background::Color(palette.primary.base.color)),
            border: Border {
                radius: radius.into(),
                width: 0.0,
                color: palette.primary.base.color,
            },
            mark_color: palette.primary.base.text,
            label_color: palette.background.base.text,
        }
    } else {
        Style {
            box_background: None,
            border: Border {
                radius: radius.into(),
                width: 2.0,
                color: palette.background.strong.color,
            },
            mark_color: palette.primary.base.text,
            label_color: palette.background.base.text,
        }
    };

    match status {
        Status::Active => base,
        Status::Hovered => {
            let overlay = Color {
                a: 0.08,
                ..base.border.color
            };
            Style {
                box_background: Some(Background::Color(match base.box_background {
                    Some(Background::Color(c)) => blend_over(c, overlay),
                    _ => overlay,
                })),
                ..base
            }
        }
        Status::Pressed => {
            let overlay = Color {
                a: 0.12,
                ..base.border.color
            };
            Style {
                box_background: Some(Background::Color(match base.box_background {
                    Some(Background::Color(c)) => blend_over(c, overlay),
                    _ => overlay,
                })),
                ..base
            }
        }
        Status::Disabled => Style {
            box_background: base.box_background.map(|_| {
                Background::Color(Color {
                    a: 0.12,
                    ..palette.background.strong.color
                })
            }),
            border: Border {
                color: Color {
                    a: 0.38,
                    ..base.border.color
                },
                ..base.border
            },
            mark_color: Color {
                a: 0.38,
                ..base.mark_color
            },
            label_color: Color {
                a: 0.38,
                ..base.label_color
            },
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
