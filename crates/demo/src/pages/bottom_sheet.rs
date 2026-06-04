use iced::widget::{column, text};
use iced_ui::bottom_sheet::BottomSheet;
use iced_ui::icon_button::{self, IconButton};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Toggle,
    Close,
}

#[derive(Default)]
pub(crate) struct BottomSheetPage {
    expanded: bool,
}

impl super::PageView for BottomSheetPage {
    type Msg = Msg;
    const LABEL: &'static str = "BottomSheet";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Toggle => self.expanded = !self.expanded,
            Msg::Close => self.expanded = false,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let host_content: Element<'_, Msg> = column![
            Text::h1("Bottom Sheet"),
            text("A panel sliding from the bottom. Modal or standard.").size(14),
            Text::h2("Modal"),
            IconButton::new(text("Toggle Sheet").size(14))
                .variant(icon_button::Variant::Filled)
                .size(140.0)
                .on_press(Msg::Toggle),
        ]
        .spacing(16)
        .padding(20)
        .into();

        BottomSheet::new(
            host_content,
            "This is the bottom sheet content. It slides up from the bottom of the screen.",
        )
        .modal(true)
        .expanded(self.expanded)
        .on_dismiss(Msg::Close)
        .drag_handle(true)
        .into()
    }
}
