//! Styling primitives for the [`Card`](crate::card::Card) widget.
//!
//! The card's rounded corners are driven entirely by
//! [`Style::border`]'s [`Radius`], which acts as the theme-level
//! "roundness" for the widget. Override a card's [`Style`] to change
//! the roundness, colors, border or shadow.
//!
//! [`Radius`]: iced::border::Radius

use iced::{Background, Border, Color, Shadow, Theme};

/// The visual variants of a [`Card`](crate::card::Card).
///
/// The variant is picked with [`Card::flat`] or [`Card::elevated`] and
/// is forwarded to the [`Catalog`] so a single style closure can return
/// different appearances for each one.
///
/// [`Card::flat`]: crate::card::Card::flat
/// [`Card::elevated`]: crate::card::Card::elevated
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    /// Bordered card with no shadow. This is the default.
    #[default]
    Flat,
    /// Card with a drop shadow. Typically rendered with a subtle or no
    /// border.
    Elevated,
}

/// The visual style of a [`Card`](crate::card::Card) in a single
/// state.
///
/// [`Style::border`]'s radius doubles as the card's "roundness" —
/// override it to change how rounded the corners appear.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background of the card. Drawn behind the background image if
    /// one is configured on the [`Card`](crate::card::Card).
    pub background: Option<Background>,
    /// Border of the card. Its [`radius`](Border::radius) drives the
    /// card's roundness — the same radius is applied to the shadow
    /// and to any raster background image.
    pub border: Border,
    /// Shadow of the card. Zeroed for the flat variant; set to a
    /// drop-shadow for the elevated variant.
    pub shadow: Shadow,
    /// Text color inherited by the card's children. `None` keeps the
    /// inherited color.
    pub text_color: Option<Color>,
}

/// A function that returns a [`Style`] for a given `Theme` and
/// [`Variant`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Variant) -> Style + 'a>;

/// A catalog of theme-driven [`Style`]s for the card.
///
/// The default class — [`Catalog::default`] — uses [`default`] to
/// derive a [`Style`] from the theme's extended palette for each
/// [`Variant`].
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] given the active theme, class and variant.
    fn style(&self, class: &Self::Class<'_>, variant: Variant) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, variant: Variant) -> Style {
        class(self, variant)
    }
}

/// The default [`Style`] of a [`Card`](crate::card::Card) for the
/// given [`Theme`] and [`Variant`].
pub fn default(theme: &Theme, variant: Variant) -> Style {
    match variant {
        Variant::Flat => flat(theme),
        Variant::Elevated => elevated(theme),
    }
}

/// The default [`Style`] of a flat, bordered
/// [`Card`](crate::card::Card).
pub fn flat(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: Some(palette.background.base.color.into()),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        shadow: Shadow::default(),
        text_color: Some(palette.background.base.text),
    }
}

/// The default [`Style`] of an elevated, shadowed
/// [`Card`](crate::card::Card).
pub fn elevated(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: Some(palette.background.base.color.into()),
        border: Border {
            radius: 8.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow {
            color: Color {
                a: 0.2,
                ..Color::BLACK
            },
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        text_color: Some(palette.background.base.text),
    }
}
