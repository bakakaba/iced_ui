use iced::widget::{column, row, text};
use iced_ui::icon_button::{self, IconButton};
use iced_ui::icons::{self, Icon};
use iced_ui::text::Text;

use crate::Element;
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
        let standard = IconButton::new(icons::icon(Icon::X).size(18)).on_press(Msg::Noop);

        let filled = IconButton::new(icons::icon(Icon::Plus).size(18))
            .variant(icon_button::Variant::Filled)
            .on_press(Msg::Noop);

        let tonal = IconButton::new(icons::icon(Icon::CircleHelp).size(18))
            .variant(icon_button::Variant::FilledTonal)
            .on_press(Msg::Noop);

        let outlined = IconButton::new(icons::icon(Icon::AlertTriangle).size(18))
            .variant(icon_button::Variant::Outlined)
            .on_press(Msg::Noop);

        let toggle = IconButton::new(icons::icon(Icon::Star).size(18))
            .variant(icon_button::Variant::Filled)
            .toggled(self.toggled)
            .on_press(Msg::Toggled);

        column![
            text("Four variants: Standard, Filled, Filled Tonal, Outlined. Supports toggle.")
                .size(14),
            Text::h2("Variants"),
            row![standard, filled, tonal, outlined, toggle].spacing(16),
            text(format!("Toggle state: {}", self.toggled)).size(12),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
