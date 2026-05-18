//! Snapshot tests for the [`iced_ui::NavigationRail`] widget.

use iced::Length;
use iced::widget::container;
use iced_test::Error;
use iced_ui::navigation_rail::{self, NavigationRail};
use iced_ui_tests::{TALL_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    Selected(usize),
}

#[test]
fn navigation_rail_default() -> Result<(), Error> {
    let rail = NavigationRail::new(Message::Selected)
        .push(navigation_rail::Destination::new("Home"))
        .push(navigation_rail::Destination::new("Search"))
        .push(navigation_rail::Destination::new("Library"))
        .push(navigation_rail::Destination::new("Settings"))
        .active(0);

    let element = container(rail).height(Length::Fixed(300.0));

    assert_snapshot::<Message>("navigation_rail_default", element, TALL_SIZE)
}
