use iced::widget::{column, text};
use iced_ui::navigation_bar::{self, NavigationBar};
use iced_ui::text::Text;
use lucide_icons::Icon;

use crate::Element;
use crate::icons::lucide;
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
            .push(navigation_bar::Destination::new("Home").icon(lucide(Icon::Home)))
            .push(navigation_bar::Destination::new("Search").icon(lucide(Icon::Search)))
            .push(navigation_bar::Destination::new("Profile").icon(lucide(Icon::User)))
            .active(0);

        column![
            Text::h1("Navigation Bar"),
            text("Bottom bar with 3-5 icon+label destinations.").size(14),
            Text::h2("Default"),
            bar,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
