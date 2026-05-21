use iced::Length;
use iced::widget::{column, container, text};
use iced_ui::navigation_rail::{self, NavigationRail};

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Noop,
}

#[derive(Default)]
pub(crate) struct NavigationRailPage;

impl super::PageView for NavigationRailPage {
    type Msg = Msg;
    const LABEL: &'static str = "NavigationRail";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let rail = NavigationRail::new(|_idx| Msg::Noop)
            .push(navigation_rail::Destination::new("Home"))
            .push(navigation_rail::Destination::new("Search"))
            .push(navigation_rail::Destination::new("Library"))
            .push(navigation_rail::Destination::new("Settings"))
            .active(0);

        column![
            text("Navigation Rail").size(20),
            text("Vertical icon+label destinations for desktop/tablet.").size(14),
            container(rail).height(Length::Fixed(300.0)),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
