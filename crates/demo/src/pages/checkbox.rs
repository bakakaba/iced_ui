use iced::widget::{column, text};
use iced_ui::checkbox::Checkbox;
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Binary(bool),
    ReadOnly(Option<bool>),
    Cyclable(Option<bool>),
}

#[derive(Default)]
pub(crate) struct CheckboxPage {
    binary: bool,
    // Both tri-state checkboxes default to the indeterminate (`None`)
    // value so the indeterminate display is visible on load.
    read_only: Option<bool>,
    cyclable: Option<bool>,
}

impl super::PageView for CheckboxPage {
    type Msg = Msg;
    const LABEL: &'static str = "Checkbox";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Binary(next) => self.binary = next,
            Msg::ReadOnly(next) => self.read_only = next,
            Msg::Cyclable(next) => self.cyclable = next,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        // A `bool` value yields a binary checkbox: clicking toggles
        // between unchecked and checked.
        let binary = Checkbox::new(self.binary)
            .label(text("Remember me").size(14))
            .on_toggle(Msg::Binary);

        // An `Option<bool>` value yields a tri-state checkbox. By
        // default the indeterminate state is read-only: it can be
        // displayed, but clicking only moves checked/unchecked (a click
        // while indeterminate selects), never back to indeterminate.
        let read_only = Checkbox::new(self.read_only)
            .label(text("Select all").size(14))
            .on_toggle(Msg::ReadOnly);

        // `.indeterminate(true)` opts into the cyclable mode: clicking
        // cycles None -> Some(true) -> Some(false) -> None, so the user
        // can reach the indeterminate ("any") value.
        let cyclable = Checkbox::new(self.cyclable)
            .label(text("Any").size(14))
            .indeterminate(true)
            .on_toggle(Msg::Cyclable);

        let disabled = Checkbox::new(Some(true)).label(text("Disabled").size(14));

        column![
            text("The value type drives behavior: bool is binary, Option<bool> is tri-state.")
                .size(14),
            Text::h2("Binary"),
            binary,
            text(format!("Value: {:?}", self.binary)).size(12),
            Text::h2("Indeterminate"),
            text("Read-only: clicking only toggles checked/unchecked; never returns to indeterminate.")
                .size(12),
            read_only,
            text(format!("Value: {:?}", self.read_only)).size(12),
            text("Cyclable: clicking also cycles through the indeterminate value.").size(12),
            cyclable,
            text(format!("Value: {:?}", self.cyclable)).size(12),
            Text::h2("Disabled"),
            disabled,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
