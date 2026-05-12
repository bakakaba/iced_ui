//! `iced_ui` — a component library built on top of [iced].
//!
//! This crate currently provides:
//!
//! - [`MenuBar`] — a horizontal top-of-app menu bar with dropdowns,
//!   nested submenus, keyboard shortcuts and a pluggable style.
//! - [`Card`] — a rounded-corner presentational container with a flat
//!   or elevated (drop-shadowed) variant, and optional background
//!   color or image.
//! - [`list::List`] — a vertical list of interactive items with
//!   hover/press feedback, suitable for navigation sidebars and
//!   selection lists.
//!
//! [iced]: https://github.com/iced-rs/iced

#![warn(missing_docs)]

pub mod card;
pub mod list;
pub mod menu;
pub mod theme;

pub use card::{Card, Variant};
pub use menu::{
    Catalog, Icon, Item, KeyBinding, Menu, MenuBar, Separator, Style, StyleFn, default, shortcuts,
};
pub use theme::{FontSize, PaddingSource, Roundness, RoundnessBase, Space, SpacingBase, Theme};
