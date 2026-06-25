use iced::widget::{column, text};
use iced_ui::checkbox::Checkbox;
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    BinaryToggled(bool),
    TriStateToggled(Option<bool>),
}

#[derive(Default)]
pub(crate) struct CheckboxPage {
    binary: bool,
    tri_state: Option<bool>,
}

impl super::PageView for CheckboxPage {
    type Msg = Msg;
    const LABEL: &'static str = "Checkbox";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::BinaryToggled(next) => self.binary = next,
            Msg::TriStateToggled(next) => self.tri_state = next,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        // A `bool` value yields a binary checkbox: clicking toggles
        // between unchecked and checked.
        let binary = Checkbox::new(self.binary)
            .label(text("Remember me").size(14))
            .on_toggle(Msg::BinaryToggled);

        // An `Option<bool>` value yields a tri-state checkbox: clicking
        // cycles None -> Some(true) -> Some(false) -> None, so the user
        // can reach the indeterminate ("any") value.
        let tri_state = Checkbox::new(self.tri_state)
            .label(text("Any").size(14))
            .on_toggle(Msg::TriStateToggled);

        let disabled = Checkbox::new(Some(true)).label(text("Disabled").size(14));

        column![
            text("The value type drives behavior: bool is binary, Option<bool> is tri-state.")
                .size(14),
            Text::h2("Binary"),
            binary,
            text(format!("Value: {:?}", self.binary)).size(12),
            Text::h2("Indeterminate"),
            tri_state,
            text(format!("Value: {:?}", self.tri_state)).size(12),
            Text::h2("Disabled"),
            disabled,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
