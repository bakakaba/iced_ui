use iced::widget::{column, text};
use iced_ui::icon_button::{self, IconButton};
use iced_ui::snackbar::Snackbar;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Show,
    Hide,
}

#[derive(Default)]
pub(crate) struct SnackbarPage {
    visible: bool,
}

impl super::PageView for SnackbarPage {
    type Msg = Msg;
    const LABEL: &'static str = "Snackbar";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Show => self.visible = true,
            Msg::Hide => self.visible = false,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let host_content: Element<'_, Msg> = column![
            text("Snackbar").size(20),
            text("Temporary notification bar at the bottom of the host.").size(14),
            IconButton::new(text("Show Snackbar").size(14))
                .variant(icon_button::Variant::Filled)
                .size(140.0)
                .on_press(Msg::Show),
        ]
        .spacing(16)
        .padding(20)
        .into();

        Snackbar::new(host_content)
            .message("Item has been archived.")
            .action("Undo", Msg::Hide)
            .on_dismiss(Msg::Hide)
            .visible(self.visible)
            .into()
    }
}
