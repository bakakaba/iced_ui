use iced::widget::{column, text};
use iced_ui::icons::{self, Icon};
use iced_ui::tabs::{Tab, Tabs};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Selected(usize),
}

#[derive(Default)]
pub(crate) struct TabsPage {
    active: usize,
}

impl super::PageView for TabsPage {
    type Msg = Msg;
    const LABEL: &'static str = "Tabs";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Selected(idx) => self.active = idx,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let tabs = Tabs::new(Msg::Selected)
            .push(Tab::new("Photos").icon(icons::icon(Icon::Image)))
            .push(Tab::new("Videos").icon(icons::icon(Icon::Video)))
            .push(Tab::new("Music").icon(icons::icon(Icon::Music)))
            .active(self.active);

        column![
            Text::h1("Tabs"),
            text("Horizontal tab row with active indicator.").size(14),
            Text::h2("Default"),
            tabs,
            text(format!("Active tab: {}", self.active)).size(12),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
