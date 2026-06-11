//! Lucide icon helpers for `iced_ui` widgets.
//!
//! This module is available when the `lucide-icons` feature is enabled.
//! It re-exports the [`lucide_icons`] crate's [`Icon`] enum and
//! font data, plus convenience helpers that produce themed
//! [`iced::widget::Text`] widgets.
//!
//! # Setup
//!
//! Consumers must register the icon font at application startup:
//!
//! ```ignore
//! iced::application(...)
//!     .font(iced_ui::icons::FONT_BYTES)
//!     .run()
//! ```
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::icons::{self, Icon};
//!
//! let close_btn = icons::close().size(18);
//! let custom = icons::icon(Icon::Heart).size(24);
//! ```

use iced::Font;
use iced::widget::text;

pub use lucide_icons::{Icon, LUCIDE_FONT_BYTES as FONT_BYTES};

/// The iced [`Font`] descriptor for the bundled Lucide icon font.
pub const FONT: Font = Font::with_name("lucide");

/// Creates a text widget rendering the given Lucide icon variant.
///
/// The returned widget uses the Lucide icon font and can be further
/// customized with `.size()`, `.color()`, etc.
pub fn icon(variant: Icon) -> iced::widget::Text<'static, crate::Theme> {
    text(char::from(variant)).font(FONT)
}

/// Close / dismiss icon (X).
pub fn close() -> iced::widget::Text<'static, crate::Theme> {
    icon(Icon::X)
}

/// Informational icon (circled "i").
pub fn info() -> iced::widget::Text<'static, crate::Theme> {
    icon(Icon::Info)
}

/// Success icon (circled checkmark).
pub fn circle_check() -> iced::widget::Text<'static, crate::Theme> {
    icon(Icon::CheckCircle2)
}

/// Warning icon (alert triangle).
pub fn triangle_alert() -> iced::widget::Text<'static, crate::Theme> {
    icon(Icon::AlertTriangle)
}

/// Error icon (circled X).
pub fn circle_x() -> iced::widget::Text<'static, crate::Theme> {
    icon(Icon::CircleX)
}
