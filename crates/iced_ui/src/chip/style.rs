//! Styling primitives for the [`Chip`](super::Chip) widget.

use iced::{Background, Border, Color, Shadow};

use crate::{Space, Theme};

/// The size of a chip.
///
/// A chip's size only selects its label font size; the pill height and
/// padding are derived from that font size and the theme's spacing, so
/// the pill always hugs its text. Chips are one step denser than
/// [`Button`](crate::Button)s.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChipSize {
    /// Small label.
    Sm,
    /// Medium label. This is the default.
    #[default]
    Md,
    /// Large label.
    Lg,
}

impl ChipSize {
    /// Returns the font size as a fraction of the given base text size.
    ///
    /// Pass the theme's base text size (e.g. `theme.text_size()`). The
    /// factors are spaced so the three sizes are clearly distinct.
    pub fn font_size(self, base: f32) -> f32 {
        match self {
            Self::Sm => base * 0.6875,
            Self::Md => base * 0.75,
            Self::Lg => base * 0.875,
        }
    }

    /// Returns the `(horizontal, vertical)` padding tokens for this
    /// size, resolved against the theme's spacing. Larger sizes get
    /// proportionally roomier padding.
    pub fn padding(self) -> (Space, Space) {
        match self {
            Self::Sm => (Space::sx(1.0), Space::sx(0.5)),
            Self::Md => (Space::sx(1.25), Space::sx(0.75)),
            Self::Lg => (Space::sx(1.5), Space::sx(1.0)),
        }
    }
}

/// The color token applied to a chip's pill fill.
///
/// When a chip is given a color, the pill is filled with the resolved
/// color and the label uses its readable contrast. When a chip has no
/// color (the default), it renders as a transparent, outlined pill.
///
/// Each variant resolves to a concrete [`Color`] from the active theme
/// at draw time.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ChipColor {
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
///
/// [`Style::border`]'s radius is overwritten by the widget at draw time
/// to produce a full pill (radius = height / 2), so a custom style
/// closure does not need to set it.
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

/// A function that returns a [`Style`] for a given theme, optional color
/// token, and interaction status.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Option<ChipColor>, Status) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Chip`](super::Chip).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class, optional color token,
    /// and status.
    fn style(&self, class: &Self::Class<'_>, color: Option<ChipColor>, status: Status) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, color: Option<ChipColor>, status: Status) -> Style {
        class(self, color, status)
    }
}

/// The default chip style.
///
/// With a [`ChipColor`], the pill is filled with the resolved color and
/// the label uses its readable contrast. Without a color, the chip is a
/// transparent, outlined pill.
pub fn default(theme: &Theme, color: Option<ChipColor>, status: Status) -> Style {
    let palette = theme.extended_palette();

    // The accent color used to derive hover/press overlays.
    let accent = match color {
        Some(token) => resolve(theme, token).0,
        None => palette.background.base.text,
    };

    let base = match color {
        Some(token) => {
            let (fill, on_fill) = resolve(theme, token);
            Style {
                background: Some(Background::Color(fill)),
                text_color: on_fill,
                border: Border::default(),
                shadow: Shadow::default(),
            }
        }
        None => Style {
            background: None,
            text_color: palette.background.base.text,
            border: Border {
                width: 1.0,
                color: palette.background.strong.color,
                ..Border::default()
            },
            shadow: Shadow::default(),
        },
    };

    match status {
        Status::Active => base,
        Status::Hovered => apply_overlay(base, accent, 0.08),
        Status::Pressed => apply_overlay(base, accent, 0.12),
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

/// Resolves a [`ChipColor`] token into its fill color and the readable
/// contrast color to use for the label on that fill.
fn resolve(theme: &Theme, color: ChipColor) -> (Color, Color) {
    let palette = theme.extended_palette();
    match color {
        ChipColor::Primary => (palette.primary.base.color, palette.primary.base.text),
        ChipColor::Secondary => (palette.secondary.base.color, palette.secondary.base.text),
        ChipColor::Success => (palette.success.base.color, palette.success.base.text),
        ChipColor::Warning => (palette.warning.base.color, palette.warning.base.text),
        ChipColor::Danger => (palette.danger.base.color, palette.danger.base.text),
        ChipColor::Foreground => (palette.background.base.text, palette.background.base.color),
        ChipColor::Information => (theme.information.base.color, theme.information.base.text),
        ChipColor::Custom(c) => (c, palette.background.base.color),
    }
}

/// Blends an `alpha`-weighted `accent` overlay over the base style's
/// background (or onto a transparent surface for outlined chips).
fn apply_overlay(base: Style, accent: Color, alpha: f32) -> Style {
    let overlay = Color { a: alpha, ..accent };
    Style {
        background: Some(Background::Color(match base.background {
            Some(Background::Color(c)) => blend_over(c, overlay),
            _ => overlay,
        })),
        ..base
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
