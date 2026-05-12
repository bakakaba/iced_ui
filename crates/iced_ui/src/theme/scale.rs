//! Typed scale tokens that resolve against the active [`Theme`].
//!
//! Spacing and roundness are expressed as **multiplication factors** of
//! a base unit defined on the [`Theme`]. A [`Space::sx(2.0)`] against
//! `Theme::spacing = 4` resolves to `8.0` logical pixels;
//! changing `spacing` to `6` rescales the same factor to `12.0`
//! without touching any call site.
//!
//! Font sizes follow a different shape — there is no `font_size_base`
//! on the [`Theme`]. A [`FontSize::Default`] resolves to iced's
//! built-in default text size; a [`FontSize::Px`] is an absolute
//! override.
//!
//! [`Space::sx(2.0)`]: Space::sx
//! [`Theme`]: crate::Theme

use iced::Padding;

/// Trait implemented by theme types that expose a spacing base unit.
///
/// Widgets in `iced_ui` that accept a [`PaddingSource`] (or otherwise
/// resolve [`Space`] tokens during layout) require their `Theme`
/// generic to implement this trait. The crate's [`Theme`] is the
/// canonical implementor; downstream apps that supply their own theme
/// type need to implement it (typically by delegating to the
/// underlying `iced_ui::Theme`).
///
/// [`Theme`]: crate::Theme
pub trait SpacingBase {
    /// Returns the spacing base unit (an integer step size).
    fn spacing(&self) -> u8;
}

/// Trait implemented by theme types that expose a roundness base
/// unit.
///
/// Widgets in `iced_ui` that resolve [`Roundness`] tokens during
/// drawing require their `Theme` generic to implement this trait.
///
/// [`Theme`]: crate::Theme
pub trait RoundnessBase {
    /// Returns the roundness base unit (an integer step size).
    fn roundness(&self) -> u8;
}

/// A spacing token, resolved against [`Theme::spacing`].
///
/// Construct with [`Space::sx`] for themed values (multiplication
/// factor) or [`Space::px`] for an absolute, unthemed escape hatch.
///
/// # Example
///
/// ```
/// use iced_ui::Space;
///
/// let s = Space::sx(3.0);
/// assert_eq!(s.resolve(4), 12.0);
/// ```
///
/// [`Theme::spacing`]: crate::Theme::spacing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Space(Repr);

/// A roundness token, resolved against [`Theme::roundness`].
///
/// Construct with [`Roundness::sx`] for themed values (multiplication
/// factor) or [`Roundness::px`] for an absolute, unthemed escape
/// hatch.
///
/// # Example
///
/// ```
/// use iced_ui::Roundness;
///
/// let r = Roundness::sx(2.0);
/// assert_eq!(r.resolve(4), 8.0);
/// ```
///
/// [`Theme::roundness`]: crate::Theme::roundness
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Roundness(Repr);

/// Internal storage shared by [`Space`] and [`Roundness`].
#[derive(Debug, Clone, Copy, PartialEq)]
enum Repr {
    /// A multiplication factor applied to the base unit.
    Sx(f32),
    /// An absolute pixel value, ignoring the base unit.
    Pixels(f32),
}

impl Space {
    /// Constructs a themed spacing token equal to `factor * base`.
    pub const fn sx(factor: f32) -> Self {
        Self(Repr::Sx(factor))
    }

    /// Constructs an absolute spacing token of `value` logical pixels.
    pub const fn px(value: f32) -> Self {
        Self(Repr::Pixels(value))
    }

    /// Resolves this spacing token against the given base unit.
    pub fn resolve(self, base: u8) -> f32 {
        match self.0 {
            Repr::Sx(factor) => factor * f32::from(base),
            Repr::Pixels(v) => v,
        }
    }
}

impl Roundness {
    /// Constructs a themed roundness token equal to `factor * base`.
    pub const fn sx(factor: f32) -> Self {
        Self(Repr::Sx(factor))
    }

    /// Constructs an absolute roundness token of `value` logical
    /// pixels.
    pub const fn px(value: f32) -> Self {
        Self(Repr::Pixels(value))
    }

    /// Resolves this roundness token against the given base unit.
    pub fn resolve(self, base: u8) -> f32 {
        match self.0 {
            Repr::Sx(factor) => factor * f32::from(base),
            Repr::Pixels(v) => v,
        }
    }
}

impl From<u8> for Space {
    fn from(n: u8) -> Self {
        Self::sx(f32::from(n))
    }
}

impl From<u8> for Roundness {
    fn from(n: u8) -> Self {
        Self::sx(f32::from(n))
    }
}

/// A text size token. Either inherits iced's built-in default text
/// size, or specifies an absolute override.
///
/// `iced_ui` does not maintain a `font_size_base` on the [`Theme`];
/// this type intentionally avoids the multiplier shape used by
/// [`Space`] and [`Roundness`] because typography looks best when
/// driven by an absolute value, not a scaled one.
///
/// [`Theme`]: crate::Theme
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontSize {
    /// Inherit the renderer's default text size.
    #[default]
    Default,
    /// Use an absolute size, in logical pixels.
    Px(f32),
}

impl FontSize {
    /// Resolves this font size, falling back to `default_size` when
    /// the variant is [`FontSize::Default`].
    pub fn resolve(self, default_size: f32) -> f32 {
        match self {
            Self::Default => default_size,
            Self::Px(v) => v,
        }
    }
}

impl From<f32> for FontSize {
    fn from(value: f32) -> Self {
        Self::Px(value)
    }
}

/// Source for a widget's padding.
///
/// Either a themed [`Space`] (uniform on all four sides, resolved
/// against the active [`Theme::spacing`]) or an absolute
/// [`iced::Padding`] (escape hatch for asymmetric layouts).
///
/// Most call sites should pass a [`Space`]; reach for an absolute
/// [`iced::Padding`] only when the four sides legitimately differ.
///
/// [`Theme::spacing`]: crate::Theme::spacing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaddingSource {
    /// Themed, uniform padding driven by the theme's spacing base.
    Space(Space),
    /// Absolute padding, in logical pixels.
    Absolute(Padding),
}

impl PaddingSource {
    /// Resolves this padding source against the given spacing base
    /// unit.
    pub fn resolve(self, spacing: u8) -> Padding {
        match self {
            Self::Space(s) => Padding::new(s.resolve(spacing)),
            Self::Absolute(p) => p,
        }
    }
}

impl From<Space> for PaddingSource {
    fn from(space: Space) -> Self {
        Self::Space(space)
    }
}

impl From<Padding> for PaddingSource {
    fn from(padding: Padding) -> Self {
        Self::Absolute(padding)
    }
}

impl From<[f32; 2]> for PaddingSource {
    fn from(value: [f32; 2]) -> Self {
        Self::Absolute(Padding::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_sx_multiplies() {
        assert_eq!(Space::sx(0.0).resolve(4), 0.0);
        assert_eq!(Space::sx(1.0).resolve(4), 4.0);
        assert_eq!(Space::sx(3.0).resolve(4), 12.0);
        assert_eq!(Space::sx(1.5).resolve(4), 6.0);
    }

    #[test]
    fn space_px_ignores_base() {
        assert_eq!(Space::px(7.5).resolve(4), 7.5);
        assert_eq!(Space::px(7.5).resolve(99), 7.5);
    }

    #[test]
    fn roundness_sx_multiplies() {
        assert_eq!(Roundness::sx(0.0).resolve(4), 0.0);
        assert_eq!(Roundness::sx(2.0).resolve(4), 8.0);
        assert_eq!(Roundness::sx(1.5).resolve(4), 6.0);
    }

    #[test]
    fn roundness_px_ignores_base() {
        assert_eq!(Roundness::px(3.5).resolve(4), 3.5);
    }

    #[test]
    fn from_u8_maps_to_sx() {
        assert_eq!(Space::from(2u8), Space::sx(2.0));
        assert_eq!(Roundness::from(2u8), Roundness::sx(2.0));
    }

    #[test]
    fn font_size_default_falls_back() {
        assert_eq!(FontSize::Default.resolve(14.0), 14.0);
        assert_eq!(FontSize::Px(20.0).resolve(14.0), 20.0);
    }

    #[test]
    fn padding_source_resolves() {
        assert_eq!(
            PaddingSource::from(Space::sx(3.0)).resolve(4),
            Padding::new(12.0),
        );
        assert_eq!(
            PaddingSource::from([1.0, 2.0]).resolve(4),
            Padding::from([1.0, 2.0]),
        );
    }
}
