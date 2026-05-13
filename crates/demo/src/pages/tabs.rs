use iced::widget::{column, text};
use iced_ui::tabs::{Tab, Tabs};

use crate::Element;
use crate::app::Demo;
use crate::message::Message;

pub(super) fn build<'a>(demo: &Demo) -> Element<'a, Message> {
    let tabs = Tabs::new(Message::TabSelected)
        .push(Tab::new("Photos"))
        .push(Tab::new("Videos"))
        .push(Tab::new("Music"))
        .active(demo.tab_active);

    column![
        text("Tabs").size(20),
        text("Horizontal tab row with active indicator.").size(14),
        tabs,
        text(format!("Active tab: {}", demo.tab_active)).size(12),
    ]
    .spacing(16)
    .padding(20)
    .into()
}
