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
use crate::state::ActionLog;

/// The top-level page message — wraps per-page messages via `Element::map()`.
#[derive(Debug, Clone)]
pub(crate) enum Message {
    // Pages with local state (mapped)
    Chip(chip::Message),
    IconButton(icon_button::Message),
    SegmentedButton(segmented_button::Message),
    Snackbar(snackbar::Message),
    BottomSheet(bottom_sheet::Message),
    Tabs(tabs::Message),
    NavigationDrawer(navigation_drawer::Message),
    Colors(colors::Message),
    Fab(fab::Message),
    // Pages that emit global messages directly
    OpenDialog,
    Noop,
}

/// Actions that pages request from the parent.
pub(crate) enum Action {
    None,
    OpenDialog,
    Log(String),
}

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

/// Holds page-local state for pages that need it.
#[derive(Debug, Default)]
pub(crate) struct PageStates {
    chip: chip::ChipPage,
    icon_button: icon_button::IconButtonPage,
    segmented_button: segmented_button::SegmentedButtonPage,
    snackbar: snackbar::SnackbarPage,
    bottom_sheet: bottom_sheet::BottomSheetPage,
    tabs: tabs::TabsPage,
    navigation_drawer: navigation_drawer::NavigationDrawerPage,
    colors: colors::ColorsPage,
}

/// Handle a page message. Returns an `Action` for the parent.
pub(crate) fn update(states: &mut PageStates, message: Message) -> Action {
    match message {
        Message::Chip(msg) => {
            states.chip.update(msg);
            Action::None
        }
        Message::IconButton(msg) => {
            states.icon_button.update(msg);
            Action::None
        }
        Message::SegmentedButton(msg) => {
            states.segmented_button.update(msg);
            Action::None
        }
        Message::Snackbar(msg) => {
            states.snackbar.update(msg);
            Action::None
        }
        Message::BottomSheet(msg) => {
            states.bottom_sheet.update(msg);
            Action::None
        }
        Message::Tabs(msg) => {
            states.tabs.update(msg);
            Action::None
        }
        Message::NavigationDrawer(msg) => match states.navigation_drawer.update(msg) {
            navigation_drawer::Action::None => Action::None,
            navigation_drawer::Action::LogAction(s) => Action::Log(s),
        },
        Message::Colors(msg) => {
            states.colors.update(msg);
            Action::None
        }
        Message::Fab(msg) => {
            let fab::Action::LogAction(s) = fab::update(msg);
            Action::Log(s)
        }
        Message::OpenDialog => Action::OpenDialog,
        Message::Noop => Action::None,
    }
}

/// Build the content for the currently active page.
pub(crate) fn view<'a>(
    page: Page,
    states: &'a PageStates,
    log: &'a ActionLog,
) -> Element<'a, Message> {
    match page {
        Page::Overview => overview::build(),
        Page::Colors => states.colors.view().map(Message::Colors),
        Page::Menu => menu::build(log),
        Page::Card => card::build(),
        Page::List => list::build(),
        Page::Divider => divider::build(),
        Page::Badge => badge::build(),
        Page::IconButton => states.icon_button.view().map(Message::IconButton),
        Page::Fab => fab::view().map(Message::Fab),
        Page::Chip => states.chip.view().map(Message::Chip),
        Page::SegmentedButton => states.segmented_button.view().map(Message::SegmentedButton),
        Page::Dialog => dialog::build(),
        Page::Snackbar => states.snackbar.view().map(Message::Snackbar),
        Page::BottomSheet => states.bottom_sheet.view().map(Message::BottomSheet),
        Page::Tabs => states.tabs.view().map(Message::Tabs),
        Page::Text => text::build(),
        Page::TopAppBar => top_app_bar::build(),
        Page::NavigationRail => navigation_rail::build(),
        Page::NavigationBar => navigation_bar::build(),
        Page::NavigationDrawer => states
            .navigation_drawer
            .view(log)
            .map(Message::NavigationDrawer),
    }
}
