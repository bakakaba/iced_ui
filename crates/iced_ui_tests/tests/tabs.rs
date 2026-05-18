//! Snapshot tests for the [`iced_ui::Tabs`] widget.

use iced_test::Error;
use iced_ui::tabs::{Tab, Tabs};
use iced_ui_tests::{WIDE_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    TabSelected(usize),
}

#[test]
fn tabs_default() -> Result<(), Error> {
    let element = Tabs::new(Message::TabSelected)
        .push(Tab::new("Photos"))
        .push(Tab::new("Videos"))
        .push(Tab::new("Music"))
        .active(0);

    assert_snapshot::<Message>("tabs_default", element, WIDE_SIZE)
}

#[test]
fn tabs_second_active() -> Result<(), Error> {
    let element = Tabs::new(Message::TabSelected)
        .push(Tab::new("Photos"))
        .push(Tab::new("Videos"))
        .push(Tab::new("Music"))
        .active(1);

    assert_snapshot::<Message>("tabs_second_active", element, WIDE_SIZE)
}
