//! Kitchen-sink demo for every `iced_ui` component.
//!
//! Currently showcases:
//!
//! - [`iced_ui::MenuBar`]: a horizontal menu bar with icons, keyboard
//!   shortcuts, separators and nested submenus.
//! - [`iced_ui::Card`]: a rounded-corner container with flat and
//!   elevated variants and optional color / image backgrounds.

use iced::advanced::image as advanced_image;
use iced::advanced::svg as advanced_svg;
use iced::widget::{Space, column, container, row, text};
use iced::{Color, Element, Fill, Length, Subscription, Task, Theme};

use iced_ui::Card;
use iced_ui::menu::{Icon, Item, KeyBinding, Menu, MenuBar, Separator};

pub fn main() -> iced::Result {
    iced::application(Demo::default, Demo::update, Demo::view)
        .title("iced_ui demo")
        .subscription(Demo::subscription)
        .theme(Theme::Dark)
        .run()
}

#[derive(Debug, Default)]
struct Demo {
    counter: u32,
    last_action: Option<String>,
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
}

impl Demo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Triggered(action) => {
                self.counter = self.counter.saturating_add(1);
                self.last_action = Some(format!("{action:?}"));
            }
        }
        Task::none()
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

        let body = container(
            column![
                text("iced_ui kitchen sink").size(28),
                menu_section,
                text("Card").size(20),
                build_card_showcase(),
            ]
            .spacing(16)
            .padding(20),
        )
        .width(Fill)
        .height(Fill);

        column![menu_bar, body].width(Fill).height(Fill).into()
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
