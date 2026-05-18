//! Snapshot tests for the [`iced_ui::Fab`] widget.

use iced::widget::{row, text};
use iced_test::Error;
use iced_ui::fab::{Fab, FabSize};
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    Pressed,
}

#[test]
fn fab_regular_default() -> Result<(), Error> {
    let element = row![Fab::new(text("+").size(24)).on_press(Message::Pressed)].padding(20);
    assert_snapshot::<Message>("fab_regular_default", element, DEFAULT_SIZE)
}

#[test]
fn fab_small() -> Result<(), Error> {
    let element = row![
        Fab::new(text("+").size(18))
            .size(FabSize::Small)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("fab_small", element, DEFAULT_SIZE)
}

#[test]
fn fab_large() -> Result<(), Error> {
    let element = row![
        Fab::new(text("+").size(36))
            .size(FabSize::Large)
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("fab_large", element, DEFAULT_SIZE)
}

#[test]
fn fab_extended() -> Result<(), Error> {
    let element = row![
        Fab::new(text("+").size(18))
            .label(text("Create").size(16))
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("fab_extended", element, DEFAULT_SIZE)
}

#[test]
fn fab_lowered() -> Result<(), Error> {
    let element = row![
        Fab::new(text("+").size(24))
            .lowered()
            .on_press(Message::Pressed)
    ]
    .padding(20);
    assert_snapshot::<Message>("fab_lowered", element, DEFAULT_SIZE)
}
