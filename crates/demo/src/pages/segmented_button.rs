use iced::widget::{column, text};
use iced_ui::segmented_button::{Segment, SegmentedButton};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Selected(usize),
}

#[derive(Default)]
pub(crate) struct SegmentedButtonPage {
    selected: usize,
}

impl super::PageView for SegmentedButtonPage {
    type Msg = Msg;
    const LABEL: &'static str = "SegmentedButton";
    const TITLE: &'static str = "Segmented Button";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Selected(idx) => self.selected = idx,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let segmented = SegmentedButton::new()
            .push(Segment::new(text("Day")), self.selected == 0)
            .push(Segment::new(text("Week")), self.selected == 1)
            .push(Segment::new(text("Month")), self.selected == 2)
            .on_press(Msg::Selected);

        column![
            text("Single-select toggle group with shared border.").size(14),
            Text::h2("Default"),
            segmented,
            text(format!("Selected: {}", self.selected)).size(12),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
