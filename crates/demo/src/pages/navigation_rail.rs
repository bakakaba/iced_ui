use iced::Length;
use iced::widget::{column, container, text};
use iced_ui::icons::{self, Icon};
use iced_ui::navigation_rail::{self, NavigationRail};
use iced_ui::text::Text;

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
            .push(navigation_rail::Destination::new("Home").icon(icons::icon(Icon::Home)))
            .push(navigation_rail::Destination::new("Search").icon(icons::icon(Icon::Search)))
            .push(navigation_rail::Destination::new("Library").icon(icons::icon(Icon::Library)))
            .push(navigation_rail::Destination::new("Settings").icon(icons::icon(Icon::Settings)))
            .active(0);

        column![
            Text::h1("Navigation Rail"),
            text("Vertical icon+label destinations for desktop/tablet.").size(14),
            Text::h2("Default"),
            container(rail).height(Length::Fixed(300.0)),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
