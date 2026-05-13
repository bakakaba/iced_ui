use iced::widget::{column, text};
use iced_ui::navigation_bar::{self, NavigationBar};

use crate::Element;
use crate::message::Message;

pub(super) fn build<'a>() -> Element<'a, Message> {
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
