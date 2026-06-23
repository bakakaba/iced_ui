//! Styling primitives for the [`Progress`](super::Progress) widget.

use crate::Theme;

pub use super::Dock;

/// A semantic color token used to tint the progress bar.
///
/// Each variant maps to a color group on the active [`Theme`]:
/// [`Info`](Color::Info) resolves to the theme's
/// [`information`](crate::Theme::information) group, while the others
/// resolve to the matching groups on the
/// [`extended_palette`](crate::Theme::extended_palette).
///
/// Defaults to [`Info`](Color::Info).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// The theme's informational color (typically cyan).
    #[default]
    Info,
    /// The theme's primary brand color.
    Primary,
    /// The theme's success color (typically green).
    Success,
    /// The theme's warning color (typically amber).
    Warning,
    /// The theme's danger/error color (typically red).
    Danger,
}

impl Color {
    /// Resolves this token to a concrete color on the given theme.
    pub fn resolve(self, theme: &Theme) -> iced::Color {
        let palette = theme.extended_palette();
        match self {
            Self::Info => theme.information.base.color,
            Self::Primary => palette.primary.base.color,
            Self::Success => palette.success.base.color,
            Self::Warning => palette.warning.base.color,
            Self::Danger => palette.danger.base.color,
        }
    }
}

/// The visual style of a [`Progress`](super::Progress) bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Color of the unfilled track behind the bar.
    pub track: iced::Color,
    /// Color of the filled bar / moving indicator.
    pub bar: iced::Color,
    /// Drop shadow (elevation) drawn behind the track. Zeroed for a
    /// flat, inline bar; set to a drop shadow when the bar is docked
    /// (or elevation is forced on via
    /// [`Progress::elevated`](super::Progress::elevated)).
    pub shadow: iced::Shadow,
}

/// A function that returns a [`Style`] for a given theme, color token,
/// and elevation flag.
///
/// `elevated` is the resolved elevation decision: `true` when the bar
/// should cast a shadow (docked by default, or forced via
/// [`Progress::elevated`](super::Progress::elevated)).
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Color, Dock, bool) -> Style + 'a>;

/// Catalog of theme-driven styles for a [`Progress`](super::Progress).
pub trait Catalog {
    /// The identifier of a particular style.
    type Class<'a>;

    /// Returns the default [`Self::Class`].
    fn default<'a>() -> Self::Class<'a>;

    /// Resolves a [`Style`] for the given class, color token, dock
    /// edge, and resolved elevation flag.
    fn style(&self, class: &Self::Class<'_>, color: Color, dock: Dock, elevated: bool) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, color: Color, dock: Dock, elevated: bool) -> Style {
        class(self, color, dock, elevated)
    }
}

/// The default progress style: the bar uses the resolved [`Color`]
/// token (defaulting to the theme's information color); the track uses
/// a muted background surface. When `elevated`, a drop shadow is added
/// whose offset points away from the docked edge.
pub fn default(theme: &Theme, color: Color, dock: Dock, elevated: bool) -> Style {
    let palette = theme.extended_palette();

    let shadow = if elevated {
        iced::Shadow {
            color: iced::Color {
                a: 0.3,
                ..iced::Color::BLACK
            },
            offset: dock.shadow_offset(),
            blur_radius: 8.0,
        }
    } else {
        iced::Shadow::default()
    };

    Style {
        track: palette.background.weak.color,
        bar: color.resolve(theme),
        shadow,
    }
}
