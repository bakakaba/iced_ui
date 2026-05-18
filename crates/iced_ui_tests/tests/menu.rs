//! Snapshot tests for the [`iced_ui::MenuBar`] widget.

use iced::Length;
use iced_test::Error;
use iced_ui::menu::{Item, KeyBinding, Menu, MenuBar, Separator};
use iced_ui_tests::{WIDE_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    New,
    Open,
    Save,
    Quit,
    Undo,
    Redo,
    About,
}

#[test]
fn menu_bar_closed() -> Result<(), Error> {
    let element = MenuBar::new()
        .width(Length::Fill)
        .push(file_menu())
        .push(edit_menu())
        .push(help_menu());

    assert_snapshot::<Message>("menu_bar_closed", element, WIDE_SIZE)
}

fn file_menu() -> Menu<Message> {
    Menu::new("File")
        .push(
            Item::new("New")
                .shortcut(KeyBinding::command('n'))
                .on_press(Message::New),
        )
        .push(
            Item::new("Open")
                .shortcut(KeyBinding::command('o'))
                .on_press(Message::Open),
        )
        .push(Separator)
        .push(
            Item::new("Save")
                .shortcut(KeyBinding::command('s'))
                .on_press(Message::Save),
        )
        .push(Separator)
        .push(
            Item::new("Quit")
                .shortcut(KeyBinding::command('q'))
                .on_press(Message::Quit),
        )
}

fn edit_menu() -> Menu<Message> {
    Menu::new("Edit")
        .push(
            Item::new("Undo")
                .shortcut(KeyBinding::command('z'))
                .on_press(Message::Undo),
        )
        .push(Item::new("Redo").on_press(Message::Redo))
}

fn help_menu() -> Menu<Message> {
    Menu::new("Help").push(Item::new("About").on_press(Message::About))
}
