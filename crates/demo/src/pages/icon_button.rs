use iced::widget::{column, row, text};
use iced_ui::icon_button::{self, IconButton};
use lucide_icons::Icon;

use crate::Element;
use crate::icons::lucide;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Toggled,
    Noop,
}

#[derive(Default)]
pub(crate) struct IconButtonPage {
    toggled: bool,
}

impl super::PageView for IconButtonPage {
    type Msg = Msg;
    const LABEL: &'static str = "IconButton";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Toggled => self.toggled = !self.toggled,
            Msg::Noop => {}
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let standard = IconButton::new(lucide(Icon::X).size(18)).on_press(Msg::Noop);

        let filled = IconButton::new(lucide(Icon::Plus).size(18))
            .variant(icon_button::Variant::Filled)
            .on_press(Msg::Noop);

        let tonal = IconButton::new(lucide(Icon::CircleHelp).size(18))
            .variant(icon_button::Variant::FilledTonal)
            .on_press(Msg::Noop);

        let outlined = IconButton::new(lucide(Icon::AlertTriangle).size(18))
            .variant(icon_button::Variant::Outlined)
            .on_press(Msg::Noop);

        let toggle = IconButton::new(lucide(Icon::Star).size(18))
            .variant(icon_button::Variant::Filled)
            .toggled(self.toggled)
            .on_press(Msg::Toggled);

        column![
            text("IconButton").size(20),
            text("Four variants: Standard, Filled, Filled Tonal, Outlined. Supports toggle.")
                .size(14),
            row![standard, filled, tonal, outlined, toggle].spacing(16),
            text(format!("Toggle state: {}", self.toggled)).size(12),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
