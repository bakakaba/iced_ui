//! Snapshot tests for the [`iced_ui::NavigationBar`] widget.

use iced_test::Error;
use iced_ui::navigation_bar::{self, NavigationBar};
use iced_ui_tests::{WIDE_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    Selected(usize),
}

#[test]
fn navigation_bar_default() -> Result<(), Error> {
    let element = NavigationBar::new(Message::Selected)
        .push(navigation_bar::Destination::new("Home"))
        .push(navigation_bar::Destination::new("Search"))
        .push(navigation_bar::Destination::new("Profile"))
        .active(0);

    assert_snapshot::<Message>("navigation_bar_default", element, WIDE_SIZE)
}

#[test]
fn navigation_bar_middle_active() -> Result<(), Error> {
    let element = NavigationBar::new(Message::Selected)
        .push(navigation_bar::Destination::new("Home"))
        .push(navigation_bar::Destination::new("Search"))
        .push(navigation_bar::Destination::new("Profile"))
        .active(1);

    assert_snapshot::<Message>("navigation_bar_middle_active", element, WIDE_SIZE)
}
