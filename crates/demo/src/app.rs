use iced::theme::Palette;
use iced::widget::{Space, checkbox, column, container, pick_list, row, scrollable, slider, text};
use iced::{Color, Length, Subscription, Task};

use iced_ui::Theme;
use iced_ui::card::Card;
use iced_ui::color_picker::ColorPicker;
use iced_ui::dialog::Dialog;
use iced_ui::list;
use iced_ui::menu::{Icon, Item, KeyBinding, Menu, MenuBar, Separator};
use iced_ui::text::Text;
use iced_ui::theme::tokens::Information;

use crate::Element;
use crate::icons::LUCIDE_FONT;
use crate::message::{Action, Message, PaletteField};
use crate::pages::{self, ActivePage, Page};
use crate::state::ActionLog;

pub(crate) struct Demo {
    pub(crate) action_log: ActionLog,
    pub(crate) theme: Theme,
    pub(crate) selected_iced: iced::Theme,
    pub(crate) sidebar_visible: bool,
    pub(crate) customize_palette: bool,
    pub(crate) custom_palette: Palette,
    pub(crate) information_color: Color,
    pub(crate) dialog_open: bool,
    pub(crate) active_page: ActivePage,
}

impl Default for Demo {
    fn default() -> Self {
        let selected_iced = iced::Theme::Dark;
        let custom_palette = selected_iced.palette();
        let information_color = Information::default_base(custom_palette.background);
        Self {
            action_log: ActionLog::default(),
            theme: Theme::from(selected_iced.clone()),
            selected_iced,
            sidebar_visible: true,
            customize_palette: false,
            custom_palette,
            information_color,
            dialog_open: false,
            active_page: ActivePage::default(),
        }
    }
}

impl Demo {
    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Triggered(action) => {
                self.action_log.record(format!("{action:?}"));
                if matches!(action, Action::ToggleSidebar) {
                    self.sidebar_visible = !self.sidebar_visible;
                }
            }
            Message::ThemeSelected(iced_theme) => {
                self.custom_palette = iced_theme.palette();
                self.information_color = Information::default_base(self.custom_palette.background);
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
            Message::PaletteFieldChanged { field, color } => {
                field.set(&mut self.custom_palette, color);
                if self.customize_palette {
                    self.refresh_colors();
                }
            }
            Message::InformationColorChanged(color) => {
                self.information_color = color;
                self.theme.refresh_information(self.information_color);
            }
            Message::RoundnessChanged(value) => {
                self.theme.roundness = value;
            }
            Message::SpacingChanged(value) => {
                self.theme.spacing = value;
            }
            Message::Navigate(page) => {
                self.active_page = ActivePage::navigate(page);
            }
            Message::CloseDialog | Message::DialogConfirmed => {
                self.dialog_open = false;
            }
            Message::Page(page_msg) => match self.active_page.update(page_msg) {
                pages::Action::None => {}
                pages::Action::OpenDialog => {
                    self.dialog_open = true;
                }
                pages::Action::Log(s) => {
                    self.action_log.set_last(s);
                }
            },
        }
        Task::none()
    }

    fn refresh_colors(&mut self) {
        self.theme.colors = if self.customize_palette {
            iced::Theme::custom("Custom".to_string(), self.custom_palette)
        } else {
            self.selected_iced.clone()
        };
        self.theme.refresh_paper();
        self.theme.refresh_information(self.information_color);
    }

    pub(crate) fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        let menu_bar = build_menu_bar(self.sidebar_visible);

        let current_page = self.active_page.page();
        let nav = build_nav_sidebar(current_page);

        let page_content = self.active_page.view(&self.action_log).map(Message::Page);

        let body = container(scrollable(page_content))
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

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        iced_ui::shortcuts(build_menu_bar(self.sidebar_visible).shortcuts())
    }
}

// -- Navigation sidebar --

fn build_nav_sidebar(current_page: Page) -> Element<'static, Message> {
    let mut nav_list = list::List::new();

    // Showcase section header
    nav_list = nav_list.push(list::Item::new(Text::h4("Showcase")));
    for &page in Page::SHOWCASE {
        let label = if page == current_page {
            text(page.label()).color(Color::WHITE)
        } else {
            text(page.label())
        };
        nav_list = nav_list.push(list::Item::new(label).on_press(Message::Navigate(page)));
    }

    // Widgets section header
    nav_list = nav_list.push(list::Item::new(Text::h4("Widgets")));
    for &page in Page::WIDGETS {
        let label = if page == current_page {
            text(page.label()).color(Color::WHITE)
        } else {
            text(page.label())
        };
        nav_list = nav_list.push(list::Item::new(label).on_press(Message::Navigate(page)));
    }

    let nav_list = nav_list
        .width(Length::Fixed(200.0))
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
        content = content.push(information_color_row(demo.information_color));
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

    row![
        text(field.label()).size(14),
        Space::new().width(Length::Fill),
        ColorPicker::new(color)
            .on_change(move |color| Message::PaletteFieldChanged { field, color }),
    ]
    .align_y(iced::Alignment::Center)
    .spacing(8)
    .into()
}

fn information_color_row(color: Color) -> Element<'static, Message> {
    row![
        text("Information").size(14),
        Space::new().width(Length::Fill),
        ColorPicker::new(color).on_change(Message::InformationColorChanged),
    ]
    .align_y(iced::Alignment::Center)
    .spacing(8)
    .into()
}

// -- Menu bar --

pub(crate) fn build_menu_bar<'a>(sidebar_visible: bool) -> MenuBar<'a, Message> {
    MenuBar::new()
        .width(Length::Fill)
        .push(file_menu())
        .push(edit_menu())
        .push(view_menu(sidebar_visible))
        .push(help_menu())
}

fn file_menu() -> Menu<Message> {
    let recent = Menu::new("Open Recent")
        .push(Item::new("project-alpha.rs").on_press(Message::Triggered(Action::OpenRecent(0))))
        .push(Item::new("notes.md").on_press(Message::Triggered(Action::OpenRecent(1))))
        .push(Item::new("legacy/main.py").on_press(Message::Triggered(Action::OpenRecent(2))));

    Menu::new("File")
        .push(
            Item::new("New")
                .icon(Icon::from_char(char::from(lucide_icons::Icon::FilePlus)).font(LUCIDE_FONT))
                .shortcut(KeyBinding::command('n'))
                .on_press(Message::Triggered(Action::New)),
        )
        .push(
            Item::new("Open\u{2026}")
                .icon(Icon::from_char(char::from(lucide_icons::Icon::FolderOpen)).font(LUCIDE_FONT))
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
