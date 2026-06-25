use iced::widget::{column, text};
use iced_ui::icons::{self, Icon};
use iced_ui::navigation_bar::{self, NavigationBar};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Noop,
}

#[derive(Default)]
pub(crate) struct NavigationBarPage;

impl super::PageView for NavigationBarPage {
    type Msg = Msg;
    const LABEL: &'static str = "NavigationBar";
    const TITLE: &'static str = "Navigation Bar";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let bar = NavigationBar::new(|_idx| Msg::Noop)
            .push(navigation_bar::Destination::new("Home").icon(icons::icon(Icon::Home)))
            .push(navigation_bar::Destination::new("Search").icon(icons::icon(Icon::Search)))
            .push(navigation_bar::Destination::new("Profile").icon(icons::icon(Icon::User)))
            .active(0);

        column![
            text("Bottom bar with 3-5 icon+label destinations.").size(14),
            Text::h2("Default"),
            bar,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
