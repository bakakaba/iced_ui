mod badge;
mod bottom_sheet;
mod card;
mod chip;
mod colors;
mod dialog;
mod divider;
mod fab;
mod icon_button;
mod list;
mod menu;
mod navigation_bar;
mod navigation_drawer;
mod navigation_rail;
mod overview;
mod segmented_button;
mod snackbar;
mod tabs;
mod text;
mod top_app_bar;

use crate::Element;
use crate::app::Demo;
use crate::message::Message;

/// The pages available in the sidebar navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum Page {
    // Showcase
    #[default]
    Overview,
    Colors,
    // Widgets
    Badge,
    BottomSheet,
    Card,
    Chip,
    Dialog,
    Divider,
    Fab,
    IconButton,
    List,
    Menu,
    NavigationBar,
    NavigationDrawer,
    NavigationRail,
    SegmentedButton,
    Snackbar,
    Tabs,
    Text,
    TopAppBar,
}

impl Page {
    pub(crate) const SHOWCASE: &[Self] = &[Self::Overview];

    pub(crate) const WIDGETS: &[Self] = &[
        Self::Badge,
        Self::BottomSheet,
        Self::Card,
        Self::Chip,
        Self::Colors,
        Self::Dialog,
        Self::Divider,
        Self::Fab,
        Self::IconButton,
        Self::List,
        Self::Menu,
        Self::NavigationBar,
        Self::NavigationDrawer,
        Self::NavigationRail,
        Self::SegmentedButton,
        Self::Snackbar,
        Self::Tabs,
        Self::Text,
        Self::TopAppBar,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Colors => "Colors",
            Self::Badge => "Badge",
            Self::BottomSheet => "BottomSheet",
            Self::Card => "Card",
            Self::Chip => "Chip",
            Self::Dialog => "Dialog",
            Self::Divider => "Divider",
            Self::Fab => "FAB",
            Self::IconButton => "IconButton",
            Self::List => "List",
            Self::Menu => "MenuBar",
            Self::NavigationBar => "NavigationBar",
            Self::NavigationDrawer => "NavDrawer",
            Self::NavigationRail => "NavigationRail",
            Self::SegmentedButton => "SegmentedButton",
            Self::Snackbar => "Snackbar",
            Self::Tabs => "Tabs",
            Self::Text => "Text",
            Self::TopAppBar => "TopAppBar",
        }
    }
}

pub(crate) fn build_page_content<'a>(demo: &'a Demo) -> Element<'a, Message> {
    match demo.page {
        Page::Overview => overview::build(demo),
        Page::Colors => colors::build(demo),
        Page::Menu => menu::build(demo),
        Page::Card => card::build(),
        Page::List => list::build(),
        Page::Divider => divider::build(),
        Page::Badge => badge::build(),
        Page::IconButton => icon_button::build(demo),
        Page::Fab => fab::build(),
        Page::Chip => chip::build(demo),
        Page::SegmentedButton => segmented_button::build(demo),
        Page::Dialog => dialog::build(),
        Page::Snackbar => snackbar::build(demo),
        Page::BottomSheet => bottom_sheet::build(demo),
        Page::Tabs => tabs::build(demo),
        Page::Text => text::build(),
        Page::TopAppBar => top_app_bar::build(),
        Page::NavigationRail => navigation_rail::build(),
        Page::NavigationBar => navigation_bar::build(),
        Page::NavigationDrawer => navigation_drawer::build(demo),
    }
}
