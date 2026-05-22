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

pub mod badge;
pub mod bottom_sheet;
pub mod button;
pub mod card;
pub mod chip;
pub mod color_picker;
pub mod dialog;
pub mod divider;
pub mod fab;
pub mod icon_button;
pub mod list;
pub mod menu;
pub mod navigation_bar;
pub mod navigation_drawer;
pub mod navigation_rail;
pub mod screen;
pub mod segmented_button;
pub mod snackbar;
pub mod tabs;
pub mod text;
pub mod theme;
pub mod top_app_bar;

pub use badge::Badge;
pub use bottom_sheet::BottomSheet;
pub use button::Button;
pub use card::{Card, Variant};
pub use chip::Chip;
pub use color_picker::ColorPicker;
pub use dialog::Dialog;
pub use divider::Divider;
pub use fab::Fab;
pub use icon_button::IconButton;
pub use menu::{
    Catalog, Icon, Item, KeyBinding, Menu, MenuBar, MenuButton, Separator, Style, StyleFn, default,
    shortcuts,
};
pub use navigation_bar::NavigationBar;
pub use navigation_drawer::NavigationDrawer;
pub use navigation_rail::NavigationRail;
pub use screen::Screen;
pub use segmented_button::{Segment, SegmentedButton};
pub use snackbar::Snackbar;
pub use tabs::Tabs;
pub use text::Text;
pub use theme::{
    FontSize, Information, PaddingSource, Paper, Roundness, RoundnessBase, Space, SpacingBase,
    Theme,
};
pub use top_app_bar::TopAppBar;
