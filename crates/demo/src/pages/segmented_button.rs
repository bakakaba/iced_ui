use iced::widget::{column, text};
use iced_ui::segmented_button::{Segment, SegmentedButton};

use crate::Element;
use crate::app::Demo;
use crate::message::Message;

pub(super) fn build<'a>(demo: &Demo) -> Element<'a, Message> {
    let segmented = SegmentedButton::new()
        .push(Segment::new(text("Day")), demo.segment_selected == 0)
        .push(Segment::new(text("Week")), demo.segment_selected == 1)
        .push(Segment::new(text("Month")), demo.segment_selected == 2)
        .on_press(Message::SegmentSelected);

    column![
        text("Segmented Button").size(20),
        text("Single-select toggle group with shared border.").size(14),
        segmented,
        text(format!("Selected: {}", demo.segment_selected)).size(12),
    ]
    .spacing(16)
    .padding(20)
    .into()
}
