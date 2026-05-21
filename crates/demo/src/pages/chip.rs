use iced::widget::{column, row, text};
use iced_ui::chip::Chip;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Toggled,
    Noop,
}

#[derive(Default)]
pub(crate) struct ChipPage {
    selected: bool,
}

impl super::PageView for ChipPage {
    type Msg = Msg;
    const LABEL: &'static str = "Chip";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Toggled => self.selected = !self.selected,
            Msg::Noop => {}
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let assist = Chip::assist(text("Add event").size(14)).on_press(Msg::Noop);

        let filter = Chip::filter(text("Vegetarian").size(14))
            .selected(self.selected)
            .on_press(Msg::Toggled);

        let input = Chip::input(text("John Doe").size(14))
            .on_press(Msg::Noop)
            .on_close(Msg::Noop);

        let suggestion = Chip::suggestion(text("Quick reply").size(14)).on_press(Msg::Noop);

        column![
            text("Chip").size(20),
            text("Assist, Filter, Input, Suggestion variants.").size(14),
            row![assist, filter, input, suggestion].spacing(12),
            text(format!("Filter chip selected: {}", self.selected)).size(12),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
