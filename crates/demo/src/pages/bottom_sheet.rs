use iced::widget::{center, column, text};
use iced_ui::bottom_sheet::BottomSheet;
use iced_ui::icon_button::{self, IconButton};
use iced_ui::screen::Screen;
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Toggle,
    Close,
    Resize(f32),
}

#[derive(Default)]
pub(crate) struct BottomSheetPage {
    expanded: bool,
    height_fraction: f32,
}

impl BottomSheetPage {
    fn effective_fraction(&self) -> f32 {
        if self.height_fraction == 0.0 {
            0.5
        } else {
            self.height_fraction
        }
    }
}

impl super::PageView for BottomSheetPage {
    type Msg = Msg;
    const LABEL: &'static str = "BottomSheet";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Toggle => self.expanded = !self.expanded,
            Msg::Close => self.expanded = false,
            Msg::Resize(f) => self.height_fraction = f,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let host_content: Element<'_, Msg> = center(
            IconButton::new(text("Toggle Sheet").size(14))
                .variant(icon_button::Variant::Filled)
                .size(140.0)
                .on_press(Msg::Toggle),
        )
        .into();

        let sheet = BottomSheet::new(
            host_content,
            "This is the bottom sheet content. It slides up from the bottom of the screen.",
        )
        .modal(true)
        .expanded(self.expanded)
        .on_dismiss(Msg::Close)
        .on_resize(Msg::Resize)
        .drag_handle(true)
        .height_fraction(self.effective_fraction());

        column![
            Text::h1("Bottom Sheet"),
            text("A panel sliding from the bottom. Modal or standard.").size(14),
            Text::h2("Modal"),
            Screen::new(sheet),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
