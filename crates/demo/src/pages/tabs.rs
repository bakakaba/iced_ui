use iced::widget::{column, text};
use iced_ui::tabs::{Tab, Tabs};

use crate::Element;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Selected(usize),
}

#[derive(Debug, Default)]
pub(super) struct TabsPage {
    active: usize,
}

impl TabsPage {
    pub(super) fn update(&mut self, message: Message) {
        match message {
            Message::Selected(idx) => self.active = idx,
        }
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        let tabs = Tabs::new(Message::Selected)
            .push(Tab::new("Photos"))
            .push(Tab::new("Videos"))
            .push(Tab::new("Music"))
            .active(self.active);

        column![
            text("Tabs").size(20),
            text("Horizontal tab row with active indicator.").size(14),
            tabs,
            text(format!("Active tab: {}", self.active)).size(12),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
