//! Lucide icon helpers for the demo app.
//!
//! The [`lucide_icons`] crate's `iced` feature provides convenience
//! functions like `icon_plus()`, but those are parameterized with
//! `iced::Theme`. Since this demo uses `iced_ui::Theme`, we need a
//! thin wrapper that constructs `iced::widget::Text` widgets bound to
//! our custom theme.
//!
//! This module demonstrates the pattern any consumer would use when
//! integrating an icon font library with a custom theme.

use iced::Font;
use iced::widget::text;
use lucide_icons::Icon;

/// The iced [`Font`] descriptor for the bundled Lucide icon font.
pub const LUCIDE_FONT: Font = Font::with_name("lucide");

/// Creates a text widget rendering the given Lucide icon.
///
/// The returned `Text` widget uses the Lucide icon font and can be
/// further customized with `.size()` before being passed to any
/// widget that accepts `impl Into<Element>`.
pub fn lucide(variant: Icon) -> iced::widget::Text<'static, iced_ui::Theme> {
    text(char::from(variant)).font(LUCIDE_FONT)
}
