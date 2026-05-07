//! Kitchen-sink demo for every `iced_ui` component.
//!
//! Currently showcases:
//!
//! - [`iced_ui::MenuBar`]: a horizontal menu bar with icons, keyboard
//!   shortcuts, separators and nested submenus.
//! - [`iced_ui::Card`]: a rounded-corner container with flat and
//!   elevated variants and optional color / image backgrounds.
//!
//! The right-hand pane exposes live theme settings: pick any built-in
//! [`iced::Theme`], or enable "Customize palette" to drive a
//! [`iced::theme::Palette`] with RGB sliders. Every change is reflected
//! immediately across the menu bar, cards and the settings pane itself.

use iced::advanced::image as advanced_image;
use iced::advanced::svg as advanced_svg;
use iced::theme::Palette;
use iced::widget::{Space, checkbox, column, container, pick_list, row, scrollable, slider, text};
use iced::{Color, Element, Fill, Length, Subscription, Task, Theme};

use iced_ui::Card;
use iced_ui::menu::{Icon, Item, KeyBinding, Menu, MenuBar, Separator};

pub fn main() -> iced::Result {
    iced::application(Demo::default, Demo::update, Demo::view)
        .title("iced_ui demo")
        .subscription(Demo::subscription)
        .theme(Demo::theme)
        .run()
}

#[derive(Debug)]
struct Demo {
    counter: u32,
    last_action: Option<String>,
    selected_theme: Theme,
    sidebar_visible: bool,
    customize_palette: bool,
    custom_palette: Palette,
}

impl Default for Demo {
    fn default() -> Self {
        let selected_theme = Theme::Dark;
        let custom_palette = selected_theme.palette();
        Self {
            counter: 0,
            last_action: None,
            selected_theme,
            sidebar_visible: true,
            customize_palette: false,
            custom_palette,
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
    ThemeSelected(Theme),
    CustomizeToggled(bool),
    PaletteChannelChanged {
        field: PaletteField,
        channel: Channel,
        value: f32,
    },
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
            Message::ThemeSelected(theme) => {
                // Seed the custom palette from the freshly selected
                // built-in so toggling "Customize palette" later starts
                // from a familiar place.
                self.custom_palette = theme.palette();
                self.selected_theme = theme;
            }
            Message::CustomizeToggled(enabled) => {
                if enabled {
                    self.custom_palette = self.selected_theme.palette();
                }
                self.customize_palette = enabled;
            }
            Message::PaletteChannelChanged {
                field,
                channel,
                value,
            } => {
                let mut color = field.get(&self.custom_palette);
                channel.set(&mut color, value);
                field.set(&mut self.custom_palette, color);
            }
        }
        Task::none()
    }

    fn theme(&self) -> Theme {
        self.active_theme()
    }

    fn active_theme(&self) -> Theme {
        if self.customize_palette {
            Theme::custom("Custom".to_string(), self.custom_palette)
        } else {
            self.selected_theme.clone()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let menu_bar = build_menu_bar();

        let status = match &self.last_action {
            Some(last) => format!("Last action: {last} (count: {})", self.counter),
            None => "Try opening a menu, activating an item, or pressing a shortcut.".to_string(),
        };

        let menu_section = column![
            text("MenuBar").size(20),
            text(status),
            row![
                text("Try keyboard shortcuts: "),
                text("Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Z, Ctrl+Shift+Z, Ctrl+Q, F1").size(14),
            ]
            .spacing(8),
        ]
        .spacing(8);

        let body = container(scrollable(
            column![
                text("iced_ui kitchen sink").size(28),
                menu_section,
                text("Card").size(20),
                build_card_showcase(),
            ]
            .spacing(16)
            .padding(20),
        ))
        .width(Fill)
        .height(Fill);

        let main_row: Element<'_, Message> = if self.sidebar_visible {
            row![body, build_settings_pane(self)].height(Fill).into()
        } else {
            body.into()
        };

        column![menu_bar, main_row].width(Fill).height(Fill).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_ui::shortcuts(build_menu_bar().shortcuts())
    }
}

fn build_menu_bar<'a>() -> MenuBar<'a, Message> {
    MenuBar::new()
        .width(Length::Fill)
        .push(file_menu())
        .push(edit_menu())
        .push(view_menu())
        .push(help_menu())
}

fn build_settings_pane(demo: &Demo) -> Element<'_, Message> {
    let theme_picker = pick_list(
        Theme::ALL,
        Some(demo.selected_theme.clone()),
        Message::ThemeSelected,
    )
    .width(Length::Fill);

    let mut content = column![
        text("Theme settings").size(18),
        column![text("Built-in theme").size(14), theme_picker].spacing(6),
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

    let pane = Card::new(scrollable(content.padding(4)))
        .width(Length::Fixed(280.0))
        .height(Length::Fill)
        .padding(16)
        .elevated();

    container(pane).padding(12).into()
}

fn palette_field_row<'a>(field: PaletteField, palette: Palette) -> Element<'a, Message> {
    let color = field.get(&palette);

    let swatch = container(Space::new())
        .width(Length::Fixed(20.0))
        .height(Length::Fixed(20.0))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..container::Style::default()
        });

    let header = row![
        text(field.label()).size(14),
        Space::new().width(Fill),
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

fn build_card_showcase<'a>() -> Element<'a, Message> {
    let flat_card = Card::new(
        column![
            text("Flat").size(18),
            text("Bordered frame with no shadow. This is the default variant.").size(14),
        ]
        .spacing(6),
    )
    .width(Length::Fixed(220.0))
    .padding(14);

    let elevated_card = Card::new(
        column![
            text("Elevated").size(18),
            text("Drop shadow, no border. Great for floating surfaces.").size(14),
        ]
        .spacing(6),
    )
    .width(Length::Fixed(220.0))
    .padding(14)
    .elevated();

    let tinted_card = Card::new(
        column![
            text("Tinted background").size(18).color(Color::WHITE),
            text("Background color overrides the theme default.")
                .size(14)
                .color(Color::WHITE),
        ]
        .spacing(6),
    )
    .width(Length::Fixed(220.0))
    .padding(14)
    .elevated()
    .background(iced::Color::from_rgb(0.18, 0.33, 0.62));

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
    .padding(14)
    .background_image(checker_handle());

    let svg_card = Card::new(
        column![
            Space::new().height(Length::Fixed(40.0)),
            text("SVG image").size(18).color(Color::WHITE),
            text("Vector backgrounds are supported too.")
                .size(14)
                .color(Color::WHITE),
        ]
        .spacing(6),
    )
    .width(Length::Fixed(220.0))
    .height(Length::Fixed(140.0))
    .padding(14)
    .elevated()
    .background_svg(gradient_svg_handle());

    row![flat_card, elevated_card, tinted_card, raster_card, svg_card]
        .spacing(16)
        .wrap()
        .into()
}

/// Generates a small procedural checker-pattern raster image so the
/// demo does not need to ship binary assets.
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

/// Inline SVG handle used by the SVG demo card.
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
            Item::new("Open…")
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
            Item::new("Save As…")
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

fn view_menu() -> Menu<Message> {
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
        .push(Item::new("Toggle Sidebar").on_press(Message::Triggered(Action::ToggleSidebar)))
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
