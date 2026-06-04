use iced::widget::{column, text};
use iced_ui::icon_button::{self, IconButton};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Open,
}

#[derive(Default)]
pub(crate) struct DialogPage;

impl super::PageView for DialogPage {
    type Msg = Msg;
    const LABEL: &'static str = "Dialog";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Open => super::Action::OpenDialog,
        }
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        column![
            Text::h1("Dialog"),
            text("Modal overlay with scrim, title, body, and action buttons.").size(14),
            Text::h2("Modal"),
            text("Press the button below to open a dialog:").size(14),
            IconButton::new(text("Open Dialog").size(14))
                .variant(icon_button::Variant::Filled)
                .size(120.0)
                .on_press(Msg::Open),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
