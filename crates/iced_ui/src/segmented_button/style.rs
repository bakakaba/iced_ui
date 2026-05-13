//! Styling primitives for the [`SegmentedButton`](super::SegmentedButton) widget.

use iced::{Background, Border, Color};

use crate::{Roundness, RoundnessBase, Theme};

/// The interaction status of a single segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SegmentStatus {
    /// Normal state.
    Active,
    /// Pointer hovering.
    Hovered,
    /// Being pressed.
    Pressed,
}

/// The visual style of a single segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentStyle {
    /// Background fill.
    pub background: Option<Background>,
    /// Label/icon color.
    pub text_color: Color,
    /// Border applied to this segment.
    pub border: Border,
}

impl Default for SegmentStyle {
    fn default() -> Self {
        Self {
            background: None,
            text_color: Color::WHITE,
            border: Border::default(),
        }
    }
}

/// A function that returns a [`SegmentStyle`] for a given theme,
/// selected state, and interaction status.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, bool, SegmentStatus) -> SegmentStyle + 'a>;

/// Catalog of theme-driven styles for a [`SegmentedButton`](super::SegmentedButton).
pub trait Catalog: RoundnessBase {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`SegmentStyle`] for the given class, selected state,
    /// and status.
    fn style(&self, class: &Self::Class<'_>, selected: bool, status: SegmentStatus)
    -> SegmentStyle;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(
        &self,
        class: &Self::Class<'_>,
        selected: bool,
        status: SegmentStatus,
    ) -> SegmentStyle {
        class(self, selected, status)
    }
}

/// The default segmented button style.
///
/// - Selected segments get a `primary.weak.color` background with `primary.weak.text`.
/// - Unselected segments get a transparent background with `background.base.text`.
/// - The entire group shares an outline border with rounded ends; internal
///   dividers are implied by the per-segment border (callers compose them via
///   the widget's draw logic which handles corner rounding per position).
pub fn default(theme: &Theme, selected: bool, status: SegmentStatus) -> SegmentStyle {
    let palette = theme.extended_palette();
    let radius = theme.radius(Roundness::sx(2.5));

    let base = if selected {
        SegmentStyle {
            background: Some(Background::Color(palette.primary.weak.color)),
            text_color: palette.primary.weak.text,
            border: Border {
                radius: radius.into(),
                width: 1.0,
                color: palette.primary.base.color,
            },
        }
    } else {
        SegmentStyle {
            background: None,
            text_color: palette.background.base.text,
            border: Border {
                radius: radius.into(),
                width: 1.0,
                color: palette.background.strong.color,
            },
        }
    };

    match status {
        SegmentStatus::Active => base,
        SegmentStatus::Hovered => {
            let hover = Color {
                a: 0.08,
                ..base.text_color
            };
            SegmentStyle {
                background: Some(Background::Color(match base.background {
                    Some(Background::Color(c)) => blend_over(c, hover),
                    _ => hover,
                })),
                ..base
            }
        }
        SegmentStatus::Pressed => {
            let press = Color {
                a: 0.12,
                ..base.text_color
            };
            SegmentStyle {
                background: Some(Background::Color(match base.background {
                    Some(Background::Color(c)) => blend_over(c, press),
                    _ => press,
                })),
                ..base
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
