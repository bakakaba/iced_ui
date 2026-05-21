use iced::widget::{column, text};
use iced_ui::navigation_bar::{self, NavigationBar};

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

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let bar = NavigationBar::new(|_idx| Msg::Noop)
            .push(navigation_bar::Destination::new("Home"))
            .push(navigation_bar::Destination::new("Search"))
            .push(navigation_bar::Destination::new("Profile"))
            .active(0);

        column![
            text("Navigation Bar").size(20),
            text("Bottom bar with 3-5 icon+label destinations.").size(14),
            bar,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
