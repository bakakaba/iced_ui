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
pub mod button;
pub mod card;
pub mod checkbox;
pub mod chip;
pub mod color_picker;
pub mod date_input;
mod date_time_core;
pub mod datetime_input;
pub mod dialog;
pub mod divider;
pub mod fab;
pub mod icon_button;
#[cfg(feature = "lucide-icons")]
pub mod icons;
pub mod list;
pub mod menu;
pub mod native;
pub mod navigation_bar;
pub mod navigation_drawer;
pub mod navigation_rail;
pub mod number_input;
pub mod position;
pub mod screen;
pub mod segmented_button;
pub mod slide_sheet;
pub mod snackbar;
pub mod tabs;
pub mod text;
pub mod text_input;
pub mod theme;
pub mod time_input;
pub mod top_app_bar;

pub use chrono;

pub use badge::Badge;
pub use button::Button;
pub use card::{Card, Variant};
pub use checkbox::Checkbox;
pub use chip::Chip;
pub use color_picker::ColorPicker;
pub use date_input::DateInput;
pub use datetime_input::DateTimeInput;
pub use dialog::Dialog;
pub use divider::Divider;
pub use fab::Fab;
pub use icon_button::IconButton;
pub use menu::{
    Catalog, Icon, Item, KeyBinding, Menu, MenuBar, MenuButton, Separator, Style, StyleFn, default,
    shortcuts,
};
pub use native::native;
pub use navigation_bar::NavigationBar;
pub use navigation_drawer::NavigationDrawer;
pub use navigation_rail::NavigationRail;
pub use number_input::NumberInput;
pub use position::Position;
pub use screen::Screen;
pub use segmented_button::{Segment, SegmentedButton};
pub use slide_sheet::SlideSheet;
pub use snackbar::{Notification, NotificationId, Notifications, Snackbar};
pub use tabs::Tabs;
pub use text::Text;
pub use text_input::TextInput;
pub use theme::{
    FontSize, FontSizeBase, Information, PaddingSource, Paper, Roundness, RoundnessBase, Space,
    SpacingBase, Theme,
};
pub use time_input::TimeInput;
pub use top_app_bar::TopAppBar;
