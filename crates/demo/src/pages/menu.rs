use iced::widget::{column, row, text};
use iced_ui::button::{Button, Variant};
use iced_ui::menu::{Entry, Item, Menu, MenuBar, MenuButton, Separator};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Action(String),
}

#[derive(Default)]
pub(crate) struct MenuPage;

impl super::PageView for MenuPage {
    type Msg = Msg;
    const LABEL: &'static str = "Menu";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Action(label) => super::Action::Log(format!("Menu: {label}")),
        }
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let menu_bar = MenuBar::with_menus(vec![
            Menu::new("File")
                .push(Item::new("New").on_press(Msg::Action("New".into())))
                .push(Item::new("Open").on_press(Msg::Action("Open".into())))
                .push(
                    Menu::new("Open Recent")
                        .push(Item::new("project.rs").on_press(Msg::Action("Recent 1".into())))
                        .push(Item::new("notes.md").on_press(Msg::Action("Recent 2".into()))),
                )
                .push(Separator)
                .push(Item::new("Save").on_press(Msg::Action("Save".into())))
                .push(Item::new("Quit").on_press(Msg::Action("Quit".into()))),
            Menu::new("Edit")
                .push(Item::new("Undo").on_press(Msg::Action("Undo".into())))
                .push(Item::new("Redo").on_press(Msg::Action("Redo".into())))
                .push(Separator)
                .push(Item::new("Cut").on_press(Msg::Action("Cut".into())))
                .push(Item::new("Copy").on_press(Msg::Action("Copy".into())))
                .push(Item::new("Paste").on_press(Msg::Action("Paste".into()))),
            Menu::new("Help").push(Item::new("About").on_press(Msg::Action("About".into()))),
        ]);

        let menu_button = MenuButton::new(
            Button::new(text("Actions")).variant(Variant::Ghost),
            vec![
                Entry::Item(Item::new("Cut").on_press(Msg::Action("Cut".into()))),
                Entry::Item(Item::new("Copy").on_press(Msg::Action("Copy".into()))),
                Entry::Separator,
                Entry::Item(Item::new("Paste").on_press(Msg::Action("Paste".into()))),
                Entry::Submenu(
                    Menu::new("More")
                        .push(Item::new("Select All").on_press(Msg::Action("Select All".into())))
                        .push(Item::new("Find").on_press(Msg::Action("Find".into()))),
                ),
            ],
        );

        column![
            Text::h1("Menu"),
            text("Dropdown menus attached to a menu bar or standalone button.").size(14),
            Text::h2("MenuBar"),
            menu_bar,
            Text::h2("MenuButton (standalone)"),
            row![menu_button].spacing(8),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }
}
