//! Kitchen-sink demo for every `iced_ui` component.
//!
//! Each component page showcases its default appearance without
//! overriding any theme-driven values (padding, spacing, roundness).

use iced::advanced::image as advanced_image;
use iced::advanced::svg as advanced_svg;
use iced::theme::Palette;
use iced::widget::{Space, checkbox, column, container, pick_list, row, scrollable, slider, text};
use iced::{Color, Length, Subscription, Task};

use iced_ui::Card;
use iced_ui::Theme;
use iced_ui::badge::Badge;
use iced_ui::bottom_sheet::BottomSheet;
use iced_ui::chip::Chip;
use iced_ui::dialog::Dialog;
use iced_ui::divider::Divider;
use iced_ui::fab::{Fab, FabSize};
use iced_ui::icon_button::{self, IconButton};
use iced_ui::list;
use iced_ui::menu::{Icon, Item, KeyBinding, Menu, MenuBar, Separator};
use iced_ui::navigation_bar::{self, NavigationBar};
use iced_ui::navigation_drawer::{DrawerItem, NavigationDrawer};
use iced_ui::navigation_rail::{self, NavigationRail};
use iced_ui::segmented_button::{Segment, SegmentedButton};
use iced_ui::snackbar::Snackbar;
use iced_ui::tabs::{Tab, Tabs};
use iced_ui::top_app_bar::TopAppBar;

/// Convenience alias: every widget in the demo's tree is themed by
/// `iced_ui::Theme`.
type Element<'a, Message> = iced::Element<'a, Message, Theme>;

pub fn main() -> iced::Result {
    iced::application(Demo::default, Demo::update, Demo::view)
        .title("iced_ui demo")
        .subscription(Demo::subscription)
        .theme(Demo::theme)
        .run()
}

/// The pages available in the sidebar navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Page {
    Badge,
    BottomSheet,
    Card,
    Chip,
    Dialog,
    Divider,
    Fab,
    IconButton,
    List,
    #[default]
    Menu,
    NavigationBar,
    NavigationDrawer,
    NavigationRail,
    SegmentedButton,
    Snackbar,
    Tabs,
    TopAppBar,
}

impl Page {
    const ALL: [Self; 17] = [
        Self::Badge,
        Self::BottomSheet,
        Self::Card,
        Self::Chip,
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
        Self::TopAppBar,
    ];

    fn label(self) -> &'static str {
        match self {
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
            Self::TopAppBar => "TopAppBar",
        }
    }
}

#[derive(Debug)]
struct Demo {
    counter: u32,
    last_action: Option<String>,
    theme: Theme,
    selected_iced: iced::Theme,
    sidebar_visible: bool,
    customize_palette: bool,
    custom_palette: Palette,
    page: Page,
    // State for interactive demos
    icon_btn_toggled: bool,
    chip_selected: bool,
    segment_selected: usize,
    dialog_open: bool,
    snackbar_visible: bool,
    bottom_sheet_expanded: bool,
    tab_active: usize,
    drawer_expanded: bool,
}

impl Default for Demo {
    fn default() -> Self {
        let selected_iced = iced::Theme::Dark;
        let custom_palette = selected_iced.palette();
        Self {
            counter: 0,
            last_action: None,
            theme: Theme::from(selected_iced.clone()),
            selected_iced,
            sidebar_visible: true,
            customize_palette: false,
            custom_palette,
            page: Page::default(),
            icon_btn_toggled: false,
            chip_selected: false,
            segment_selected: 0,
            dialog_open: false,
            snackbar_visible: false,
            bottom_sheet_expanded: false,
            tab_active: 0,
            drawer_expanded: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum PaletteField {
    Background,
    Text,
    Primary,
    Success,
    Warning,
    Danger,
}

impl PaletteField {
    const ALL: [Self; 6] = [
        Self::Background,
        Self::Text,
        Self::Primary,
        Self::Success,
        Self::Warning,
        Self::Danger,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Background => "Background",
            Self::Text => "Text",
            Self::Primary => "Primary",
            Self::Success => "Success",
            Self::Warning => "Warning",
            Self::Danger => "Danger",
        }
    }

    fn get(self, palette: &Palette) -> Color {
        match self {
            Self::Background => palette.background,
            Self::Text => palette.text,
            Self::Primary => palette.primary,
            Self::Success => palette.success,
            Self::Warning => palette.warning,
            Self::Danger => palette.danger,
        }
    }

    fn set(self, palette: &mut Palette, color: Color) {
        match self {
            Self::Background => palette.background = color,
            Self::Text => palette.text = color,
            Self::Primary => palette.primary = color,
            Self::Success => palette.success = color,
            Self::Warning => palette.warning = color,
            Self::Danger => palette.danger = color,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Channel {
    R,
    G,
    B,
}

impl Channel {
    fn label(self) -> &'static str {
        match self {
            Self::R => "R",
            Self::G => "G",
            Self::B => "B",
        }
    }

    fn get(self, color: Color) -> f32 {
        match self {
            Self::R => color.r,
            Self::G => color.g,
            Self::B => color.b,
        }
    }

    fn set(self, color: &mut Color, value: f32) {
        match self {
            Self::R => color.r = value,
            Self::G => color.g = value,
            Self::B => color.b = value,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Action {
    New,
    Open,
    OpenRecent(u8),
    Save,
    SaveAs,
    Quit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleSidebar,
    About,
}

#[derive(Debug, Clone)]
enum Message {
    Triggered(Action),
    ThemeSelected(iced::Theme),
    CustomizeToggled(bool),
    PaletteChannelChanged {
        field: PaletteField,
        channel: Channel,
        value: f32,
    },
    RoundnessChanged(u8),
    SpacingChanged(u8),
    Navigate(Page),
    // Interactive demo messages
    IconButtonToggled,
    ChipToggled,
    SegmentSelected(usize),
    OpenDialog,
    CloseDialog,
    DialogConfirmed,
    ShowSnackbar,
    HideSnackbar,
    ToggleBottomSheet,
    CloseBottomSheet,
    TabSelected(usize),
    ToggleDrawer,
    CloseDrawer,
    DrawerItemSelected(usize),
    FabPressed,
    Noop,
}

impl Demo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Triggered(action) => {
                self.counter = self.counter.saturating_add(1);
                self.last_action = Some(format!("{action:?}"));
                if matches!(action, Action::ToggleSidebar) {
                    self.sidebar_visible = !self.sidebar_visible;
                }
            }
            Message::ThemeSelected(iced_theme) => {
                self.custom_palette = iced_theme.palette();
                self.selected_iced = iced_theme;
                self.refresh_colors();
            }
            Message::CustomizeToggled(enabled) => {
                if enabled {
                    self.custom_palette = self.selected_iced.palette();
                }
                self.customize_palette = enabled;
                self.refresh_colors();
            }
            Message::PaletteChannelChanged {
                field,
                channel,
                value,
            } => {
                let mut color = field.get(&self.custom_palette);
                channel.set(&mut color, value);
                field.set(&mut self.custom_palette, color);
                if self.customize_palette {
                    self.refresh_colors();
                }
            }
            Message::RoundnessChanged(value) => {
                self.theme.roundness = value;
            }
            Message::SpacingChanged(value) => {
                self.theme.spacing = value;
            }
            Message::Navigate(page) => {
                self.page = page;
            }
            Message::IconButtonToggled => {
                self.icon_btn_toggled = !self.icon_btn_toggled;
            }
            Message::ChipToggled => {
                self.chip_selected = !self.chip_selected;
            }
            Message::SegmentSelected(idx) => {
                self.segment_selected = idx;
            }
            Message::OpenDialog => {
                self.dialog_open = true;
            }
            Message::CloseDialog | Message::DialogConfirmed => {
                self.dialog_open = false;
            }
            Message::ShowSnackbar => {
                self.snackbar_visible = true;
            }
            Message::HideSnackbar => {
                self.snackbar_visible = false;
            }
            Message::ToggleBottomSheet => {
                self.bottom_sheet_expanded = !self.bottom_sheet_expanded;
            }
            Message::CloseBottomSheet => {
                self.bottom_sheet_expanded = false;
            }
            Message::TabSelected(idx) => {
                self.tab_active = idx;
            }
            Message::ToggleDrawer => {
                self.drawer_expanded = !self.drawer_expanded;
            }
            Message::CloseDrawer => {
                self.drawer_expanded = false;
            }
            Message::DrawerItemSelected(idx) => {
                self.last_action = Some(format!("Drawer item {idx}"));
                self.drawer_expanded = false;
            }
            Message::FabPressed => {
                self.last_action = Some("FAB pressed".to_string());
            }
            Message::Noop => {}
        }
        Task::none()
    }

    fn refresh_colors(&mut self) {
        self.theme.colors = if self.customize_palette {
            iced::Theme::custom("Custom".to_string(), self.custom_palette)
        } else {
            self.selected_iced.clone()
        };
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn view(&self) -> Element<'_, Message> {
        let menu_bar = build_menu_bar(self.sidebar_visible);

        let nav = build_nav_sidebar(self.page);

        let body = container(scrollable(self.build_page_content()))
            .width(Length::Fill)
            .height(Length::Fill);

        let main_row: Element<'_, Message> = if self.sidebar_visible {
            row![nav, body, build_settings_pane(self)]
                .height(Length::Fill)
                .into()
        } else {
            row![nav, body].height(Length::Fill).into()
        };

        let content: Element<'_, Message> = column![menu_bar, main_row]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        // Wrap in Dialog if needed
        let content = Dialog::new(content)
            .title("Confirm Action")
            .body("Are you sure you want to proceed? This action cannot be undone.")
            .confirm("Confirm", Message::DialogConfirmed)
            .dismiss("Cancel", Message::CloseDialog)
            .on_scrim_press(Message::CloseDialog)
            .open(self.dialog_open);

        content.into()
    }

    fn build_page_content(&self) -> Element<'_, Message> {
        match self.page {
            Page::Menu => build_menu_page(self),
            Page::Card => build_card_page(),
            Page::List => build_list_page(),
            Page::Divider => build_divider_page(),
            Page::Badge => build_badge_page(),
            Page::IconButton => build_icon_button_page(self),
            Page::Fab => build_fab_page(),
            Page::Chip => build_chip_page(self),
            Page::SegmentedButton => build_segmented_button_page(self),
            Page::Dialog => build_dialog_page(),
            Page::Snackbar => build_snackbar_page(self),
            Page::BottomSheet => build_bottom_sheet_page(self),
            Page::Tabs => build_tabs_page(self),
            Page::TopAppBar => build_top_app_bar_page(),
            Page::NavigationRail => build_navigation_rail_page(),
            Page::NavigationBar => build_navigation_bar_page(),
            Page::NavigationDrawer => build_navigation_drawer_page(self),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_ui::shortcuts(build_menu_bar(self.sidebar_visible).shortcuts())
    }
}

// -- Navigation sidebar --

fn build_nav_sidebar(current_page: Page) -> Element<'static, Message> {
    let mut nav_list = list::List::new();

    for page in Page::ALL {
        let label = if page == current_page {
            text(page.label()).size(13).color(Color::WHITE)
        } else {
            text(page.label()).size(13)
        };
        let item = list::Item::new(label).on_press(Message::Navigate(page));
        nav_list = nav_list.push(item);
    }

    let nav_list = nav_list
        .width(Length::Fixed(140.0))
        .style(move |theme: &Theme, status| nav_item_style(theme, status, current_page));

    container(scrollable(nav_list)).height(Length::Fill).into()
}

fn nav_item_style(theme: &Theme, status: list::Status, _current_page: Page) -> list::ItemStyle {
    let palette = theme.extended_palette();
    match status {
        list::Status::Active => list::ItemStyle {
            background: None,
            border: iced::Border::default(),
            text_color: None,
        },
        list::Status::Hovered => list::ItemStyle {
            background: Some(iced::Background::Color(palette.background.weak.color)),
            border: iced::Border::default(),
            text_color: None,
        },
        list::Status::Pressed => list::ItemStyle {
            background: Some(iced::Background::Color(palette.primary.weak.color)),
            border: iced::Border::default(),
            text_color: None,
        },
    }
}

// -- Page content builders --

fn build_menu_page<'a>(demo: &Demo) -> Element<'a, Message> {
    let status = match &demo.last_action {
        Some(last) => format!("Last action: {last} (count: {})", demo.counter),
        None => "Try opening a menu, activating an item, or pressing a shortcut.".to_string(),
    };

    column![
        text("MenuBar").size(20),
        text(status),
        row![
            text("Try keyboard shortcuts: "),
            text("Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Z, Ctrl+Shift+Z, Ctrl+Q, F1").size(14),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .padding(20)
    .into()
}

fn build_card_page<'a>() -> Element<'a, Message> {
    let flat_card = Card::new(
        column![
            text("Flat").size(18),
            text("Bordered frame with no shadow.").size(14),
        ]
        .spacing(6),
    )
    .width(Length::Fixed(220.0));

    let elevated_card = Card::new(
        column![
            text("Elevated").size(18),
            text("Drop shadow, no border.").size(14),
        ]
        .spacing(6),
    )
    .width(Length::Fixed(220.0))
    .elevated();

    let raster_card = Card::new(
        column![
            Space::new().height(Length::Fixed(40.0)),
            text("Raster image").size(18).color(Color::WHITE),
            text("Rounded corners clip the image.")
                .size(14)
                .color(Color::WHITE),
        ]
        .spacing(6),
    )
    .width(Length::Fixed(220.0))
    .height(Length::Fixed(140.0))
    .background_image(checker_handle());

    let svg_card = Card::new(
        column![
            Space::new().height(Length::Fixed(40.0)),
            text("SVG image").size(18).color(Color::WHITE),
            text("Vector backgrounds supported.")
                .size(14)
                .color(Color::WHITE),
        ]
        .spacing(6),
    )
    .width(Length::Fixed(220.0))
    .height(Length::Fixed(140.0))
    .elevated()
    .background_svg(gradient_svg_handle());

    column![
        text("Card").size(20),
        row![flat_card, elevated_card, raster_card, svg_card]
            .spacing(16)
            .wrap(),
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_list_page<'a>() -> Element<'a, Message> {
    let example_list = list::List::new()
        .push(list::Item::new(text("Apple")))
        .push(list::Item::new(text("Banana")))
        .push(list::Item::new(text("Cherry")))
        .push(list::Item::new(text("Dragonfruit")))
        .push(list::Item::new(text("Elderberry")))
        .width(Length::Fixed(200.0));

    column![
        text("List").size(20),
        text("A vertical list of interactive items with hover/press feedback.").size(14),
        example_list,
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_divider_page<'a>() -> Element<'a, Message> {
    column![
        text("Divider").size(20),
        text("Horizontal and vertical separators with optional insets.").size(14),
        text("Full width:").size(14),
        Divider::horizontal(),
        text("With inset:").size(14),
        Divider::horizontal().inset(iced_ui::Space::sx(4.0)),
        row![
            text("Vertical:").size(14),
            Divider::vertical(),
            text("Between content").size(14),
        ]
        .spacing(8)
        .height(Length::Fixed(40.0)),
    ]
    .spacing(12)
    .padding(20)
    .into()
}

fn build_badge_page<'a>() -> Element<'a, Message> {
    let dot_badge = Badge::dot(text("Mail").size(16));
    let count_badge = Badge::count(text("Inbox").size(16), 5);
    let large_count = Badge::count(text("Notifications").size(16), 1234).max(999);

    column![
        text("Badge").size(20),
        text("Small dot or count indicator overlaid on content.").size(14),
        row![dot_badge, count_badge, large_count].spacing(32),
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_icon_button_page<'a>(demo: &Demo) -> Element<'a, Message> {
    let standard = IconButton::new(text("X").size(18)).on_press(Message::Noop);

    let filled = IconButton::new(text("+").size(18))
        .variant(icon_button::Variant::Filled)
        .on_press(Message::Noop);

    let tonal = IconButton::new(text("?").size(18))
        .variant(icon_button::Variant::FilledTonal)
        .on_press(Message::Noop);

    let outlined = IconButton::new(text("!").size(18))
        .variant(icon_button::Variant::Outlined)
        .on_press(Message::Noop);

    let toggle = IconButton::new(text("*").size(18))
        .variant(icon_button::Variant::Filled)
        .toggled(demo.icon_btn_toggled)
        .on_press(Message::IconButtonToggled);

    column![
        text("IconButton").size(20),
        text("Four variants: Standard, Filled, Filled Tonal, Outlined. Supports toggle.").size(14),
        row![standard, filled, tonal, outlined, toggle].spacing(16),
        text(format!("Toggle state: {}", demo.icon_btn_toggled)).size(12),
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_fab_page<'a>() -> Element<'a, Message> {
    let small_fab = Fab::new(text("+").size(18))
        .size(FabSize::Small)
        .on_press(Message::FabPressed);

    let regular_fab = Fab::new(text("+").size(24)).on_press(Message::FabPressed);

    let large_fab = Fab::new(text("+").size(36))
        .size(FabSize::Large)
        .on_press(Message::FabPressed);

    let extended_fab = Fab::new(text("+").size(18))
        .label(text("Create").size(16))
        .on_press(Message::FabPressed);

    let lowered_fab = Fab::new(text("+").size(24))
        .lowered()
        .on_press(Message::FabPressed);

    column![
        text("FAB (Floating Action Button)").size(20),
        text("Small, Regular, Large, Extended, and Lowered variants.").size(14),
        row![small_fab, regular_fab, large_fab, extended_fab, lowered_fab].spacing(16),
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_chip_page<'a>(demo: &Demo) -> Element<'a, Message> {
    let assist = Chip::assist(text("Add event").size(14)).on_press(Message::Noop);

    let filter = Chip::filter(text("Vegetarian").size(14))
        .selected(demo.chip_selected)
        .on_press(Message::ChipToggled);

    let input = Chip::input(text("John Doe").size(14))
        .on_press(Message::Noop)
        .on_close(Message::Noop);

    let suggestion = Chip::suggestion(text("Quick reply").size(14)).on_press(Message::Noop);

    column![
        text("Chip").size(20),
        text("Assist, Filter, Input, Suggestion variants.").size(14),
        row![assist, filter, input, suggestion].spacing(12),
        text(format!("Filter chip selected: {}", demo.chip_selected)).size(12),
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_segmented_button_page<'a>(demo: &Demo) -> Element<'a, Message> {
    let segmented = SegmentedButton::new()
        .push(Segment::new(text("Day")), demo.segment_selected == 0)
        .push(Segment::new(text("Week")), demo.segment_selected == 1)
        .push(Segment::new(text("Month")), demo.segment_selected == 2)
        .on_press(Message::SegmentSelected);

    column![
        text("Segmented Button").size(20),
        text("Single-select toggle group with shared border.").size(14),
        segmented,
        text(format!("Selected: {}", demo.segment_selected)).size(12),
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_dialog_page<'a>() -> Element<'a, Message> {
    column![
        text("Dialog").size(20),
        text("Modal overlay with scrim, title, body, and action buttons.").size(14),
        text("Press the button below to open a dialog:").size(14),
        IconButton::new(text("Open Dialog").size(14))
            .variant(icon_button::Variant::Filled)
            .size(120.0)
            .on_press(Message::OpenDialog),
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_snackbar_page<'a>(demo: &Demo) -> Element<'a, Message> {
    let host_content: Element<'_, Message> = column![
        text("Snackbar").size(20),
        text("Temporary notification bar at the bottom of the host.").size(14),
        IconButton::new(text("Show Snackbar").size(14))
            .variant(icon_button::Variant::Filled)
            .size(140.0)
            .on_press(Message::ShowSnackbar),
    ]
    .spacing(16)
    .padding(20)
    .into();

    Snackbar::new(host_content)
        .message("Item has been archived.")
        .action("Undo", Message::HideSnackbar)
        .on_dismiss(Message::HideSnackbar)
        .visible(demo.snackbar_visible)
        .into()
}

fn build_bottom_sheet_page<'a>(demo: &Demo) -> Element<'a, Message> {
    let host_content: Element<'_, Message> = column![
        text("Bottom Sheet").size(20),
        text("A panel sliding from the bottom. Modal or standard.").size(14),
        IconButton::new(text("Toggle Sheet").size(14))
            .variant(icon_button::Variant::Filled)
            .size(140.0)
            .on_press(Message::ToggleBottomSheet),
    ]
    .spacing(16)
    .padding(20)
    .into();

    BottomSheet::new(
        host_content,
        "This is the bottom sheet content. It slides up from the bottom of the screen.",
    )
    .modal(true)
    .expanded(demo.bottom_sheet_expanded)
    .on_dismiss(Message::CloseBottomSheet)
    .drag_handle(true)
    .into()
}

fn build_tabs_page<'a>(demo: &Demo) -> Element<'a, Message> {
    let tabs = Tabs::new(Message::TabSelected)
        .push(Tab::new("Photos"))
        .push(Tab::new("Videos"))
        .push(Tab::new("Music"))
        .active(demo.tab_active);

    column![
        text("Tabs").size(20),
        text("Horizontal tab row with active indicator.").size(14),
        tabs,
        text(format!("Active tab: {}", demo.tab_active)).size(12),
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_top_app_bar_page<'a>() -> Element<'a, Message> {
    let nav_icon: Element<'_, Message> = text("<").size(20).into();
    let action1: Element<'_, Message> = text("S").size(16).into();
    let action2: Element<'_, Message> = text("?").size(16).into();

    let app_bar = TopAppBar::new("Page Title")
        .navigation_icon(nav_icon)
        .action(action1)
        .action(action2);

    column![
        text("Top App Bar").size(20),
        text("Title bar with navigation icon, title, and action icons.").size(14),
        app_bar,
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_navigation_rail_page<'a>() -> Element<'a, Message> {
    let rail = NavigationRail::new(|_idx| Message::Noop)
        .push(navigation_rail::Destination::new("Home"))
        .push(navigation_rail::Destination::new("Search"))
        .push(navigation_rail::Destination::new("Library"))
        .push(navigation_rail::Destination::new("Settings"))
        .active(0);

    column![
        text("Navigation Rail").size(20),
        text("Vertical icon+label destinations for desktop/tablet.").size(14),
        container(rail).height(Length::Fixed(300.0)),
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_navigation_bar_page<'a>() -> Element<'a, Message> {
    let bar = NavigationBar::new(|_idx| Message::Noop)
        .push(navigation_bar::Destination::new("Home"))
        .push(navigation_bar::Destination::new("Search"))
        .push(navigation_bar::Destination::new("Profile"))
        .active(0);

    column![
        text("Navigation Bar").size(20),
        text("Bottom bar with 3-5 icon+label destinations.").size(14),
        bar,
    ]
    .spacing(16)
    .padding(20)
    .into()
}

fn build_navigation_drawer_page<'a>(demo: &Demo) -> Element<'a, Message> {
    let host: Element<'_, Message> = column![
        text("Navigation Drawer").size(20),
        text("Side panel with destinations. Modal with scrim.").size(14),
        IconButton::new(text("Toggle Drawer").size(14))
            .variant(icon_button::Variant::Filled)
            .size(140.0)
            .on_press(Message::ToggleDrawer),
        if let Some(last) = &demo.last_action {
            text(format!("Last: {last}")).size(12)
        } else {
            text("").size(12)
        },
    ]
    .spacing(16)
    .padding(20)
    .into();

    NavigationDrawer::new(host)
        .push(DrawerItem::header("Navigation"))
        .push(DrawerItem::destination("Home"))
        .push(DrawerItem::destination("Profile"))
        .push(DrawerItem::destination("Settings"))
        .push(DrawerItem::divider())
        .push(DrawerItem::destination("Help"))
        .active(0)
        .modal(true)
        .expanded(demo.drawer_expanded)
        .on_dismiss(Message::CloseDrawer)
        .on_select(Message::DrawerItemSelected)
        .into()
}

// -- Menu bar --

fn build_menu_bar<'a>(sidebar_visible: bool) -> MenuBar<'a, Message> {
    MenuBar::new()
        .width(Length::Fill)
        .push(file_menu())
        .push(edit_menu())
        .push(view_menu(sidebar_visible))
        .push(help_menu())
}

// -- Settings pane --

fn build_settings_pane(demo: &Demo) -> Element<'_, Message> {
    let theme_picker = pick_list(
        iced::Theme::ALL,
        Some(demo.selected_iced.clone()),
        Message::ThemeSelected,
    )
    .width(Length::Fill);

    let mut content = column![
        text("Theme settings").size(18),
        column![text("Colors (built-in)").size(14), theme_picker].spacing(6),
        checkbox(demo.customize_palette)
            .label("Customize palette")
            .on_toggle(Message::CustomizeToggled),
    ]
    .spacing(14);

    if demo.customize_palette {
        let mut editor = column![text("Palette").size(16)].spacing(12);
        for field in PaletteField::ALL {
            editor = editor.push(palette_field_row(field, demo.custom_palette));
        }
        content = content.push(editor);
    }

    content = content.push(base_slider(
        "Roundness",
        demo.theme.roundness,
        0..=24,
        Message::RoundnessChanged,
    ));
    content = content.push(base_slider(
        "Spacing",
        demo.theme.spacing,
        0..=24,
        Message::SpacingChanged,
    ));

    let pane = Card::new(scrollable(content.padding(4)))
        .width(Length::Fixed(260.0))
        .height(Length::Fill)
        .padding(iced_ui::Space::sx(4.0))
        .elevated();

    container(pane).padding(12).into()
}

fn base_slider<'a>(
    label: &'a str,
    value: u8,
    range: std::ops::RangeInclusive<u8>,
    on_change: impl Fn(u8) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        row![
            text(label).size(14),
            Space::new().width(Length::Fill),
            text(format!("{value}")).size(12),
        ]
        .align_y(iced::Alignment::Center),
        slider(range, value, on_change),
    ]
    .spacing(4)
    .into()
}

fn palette_field_row<'a>(field: PaletteField, palette: Palette) -> Element<'a, Message> {
    let color = field.get(&palette);

    let swatch = container(Space::new())
        .width(Length::Fixed(20.0))
        .height(Length::Fixed(20.0))
        .style(move |_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..iced::widget::container::Style::default()
        });

    let header = row![
        text(field.label()).size(14),
        Space::new().width(Length::Fill),
        swatch
    ]
    .align_y(iced::Alignment::Center)
    .spacing(8);

    let mut group = column![header].spacing(4);
    for channel in [Channel::R, Channel::G, Channel::B] {
        let value = channel.get(color);
        let label = format!("{} {:.2}", channel.label(), value);
        group = group.push(
            row![
                text(label).size(12).width(Length::Fixed(48.0)),
                slider(0.0..=1.0, value, move |v| Message::PaletteChannelChanged {
                    field,
                    channel,
                    value: v,
                })
                .step(0.01_f32),
            ]
            .align_y(iced::Alignment::Center)
            .spacing(8),
        );
    }
    group.into()
}

// -- Card showcase helpers --

fn checker_handle() -> advanced_image::Handle {
    const W: u32 = 64;
    const H: u32 = 64;
    let mut pixels = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            let on = ((x / 8) + (y / 8)) % 2 == 0;
            let (r, g, b) = if on {
                (0x22u8, 0x5f, 0x8c)
            } else {
                (0x40u8, 0x8f, 0xc2)
            };
            pixels.extend_from_slice(&[r, g, b, 0xff]);
        }
    }
    advanced_image::Handle::from_rgba(W, H, pixels)
}

fn gradient_svg_handle() -> advanced_svg::Handle {
    const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 120" preserveAspectRatio="none">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#8e2de2"/>
      <stop offset="100%" stop-color="#4a00e0"/>
    </linearGradient>
  </defs>
  <rect width="200" height="120" fill="url(#g)"/>
  <circle cx="160" cy="30" r="22" fill="#ffffff" fill-opacity="0.25"/>
  <circle cx="40" cy="90" r="14" fill="#ffffff" fill-opacity="0.15"/>
</svg>"##;
    advanced_svg::Handle::from_memory(SVG.to_vec())
}

// -- Menus --

fn file_menu() -> Menu<Message> {
    let recent = Menu::new("Open Recent")
        .push(Item::new("project-alpha.rs").on_press(Message::Triggered(Action::OpenRecent(0))))
        .push(Item::new("notes.md").on_press(Message::Triggered(Action::OpenRecent(1))))
        .push(Item::new("legacy/main.py").on_press(Message::Triggered(Action::OpenRecent(2))));

    Menu::new("File")
        .push(
            Item::new("New")
                .icon(Icon::from_text("+"))
                .shortcut(KeyBinding::command('n'))
                .on_press(Message::Triggered(Action::New)),
        )
        .push(
            Item::new("Open\u{2026}")
                .icon(Icon::from_text("O"))
                .shortcut(KeyBinding::command('o'))
                .on_press(Message::Triggered(Action::Open)),
        )
        .push(recent)
        .push(Separator)
        .push(
            Item::new("Save")
                .shortcut(KeyBinding::command('s'))
                .on_press(Message::Triggered(Action::Save)),
        )
        .push(
            Item::new("Save As\u{2026}")
                .shortcut(KeyBinding::command('s').shift())
                .on_press(Message::Triggered(Action::SaveAs)),
        )
        .push(Separator)
        .push(
            Item::new("Quit")
                .shortcut(KeyBinding::command('q'))
                .on_press(Message::Triggered(Action::Quit)),
        )
}

fn edit_menu() -> Menu<Message> {
    Menu::new("Edit")
        .push(
            Item::new("Undo")
                .shortcut(KeyBinding::command('z'))
                .on_press(Message::Triggered(Action::Undo)),
        )
        .push(
            Item::new("Redo")
                .shortcut(KeyBinding::command('z').shift())
                .on_press(Message::Triggered(Action::Redo)),
        )
        .push(Separator)
        .push(
            Item::new("Cut")
                .shortcut(KeyBinding::command('x'))
                .on_press(Message::Triggered(Action::Cut)),
        )
        .push(
            Item::new("Copy")
                .shortcut(KeyBinding::command('c'))
                .on_press(Message::Triggered(Action::Copy)),
        )
        .push(
            Item::new("Paste")
                .shortcut(KeyBinding::command('v'))
                .on_press(Message::Triggered(Action::Paste)),
        )
}

fn view_menu(sidebar_visible: bool) -> Menu<Message> {
    let zoom = Menu::new("Zoom")
        .push(
            Item::new("Zoom In")
                .shortcut(KeyBinding::command('='))
                .on_press(Message::Triggered(Action::ZoomIn)),
        )
        .push(
            Item::new("Zoom Out")
                .shortcut(KeyBinding::command('-'))
                .on_press(Message::Triggered(Action::ZoomOut)),
        )
        .push(
            Item::new("Reset Zoom")
                .shortcut(KeyBinding::command('0'))
                .on_press(Message::Triggered(Action::ZoomReset)),
        );

    Menu::new("View")
        .push(zoom)
        .push(Separator)
        .push(
            Item::new("Toggle Sidebar")
                .checked(sidebar_visible)
                .on_press(Message::Triggered(Action::ToggleSidebar)),
        )
        .push(Item::new("Unavailable").enabled(false))
}

fn help_menu() -> Menu<Message> {
    Menu::new("Help").push(
        Item::new("About")
            .shortcut(KeyBinding::new(
                iced::keyboard::Modifiers::empty(),
                iced::keyboard::Key::Named(iced::keyboard::key::Named::F1),
            ))
            .on_press(Message::Triggered(Action::About)),
    )
}
