use iced::widget::{column, text};
use iced_ui::segmented_button::{Segment, SegmentedButton};

use crate::Element;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Selected(usize),
}

#[derive(Debug, Default)]
pub(super) struct SegmentedButtonPage {
    selected: usize,
}

impl SegmentedButtonPage {
    pub(super) fn update(&mut self, message: Message) {
        match message {
            Message::Selected(idx) => self.selected = idx,
        }
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        let segmented = SegmentedButton::new()
            .push(Segment::new(text("Day")), self.selected == 0)
            .push(Segment::new(text("Week")), self.selected == 1)
            .push(Segment::new(text("Month")), self.selected == 2)
            .on_press(Message::Selected);

        column![
            text("Segmented Button").size(20),
            text("Single-select toggle group with shared border.").size(14),
            segmented,
            text(format!("Selected: {}", self.selected)).size(12),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
