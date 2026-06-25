use iced::Length;
use iced::widget::{column, row, text};
use iced_ui::badge::Badge;
use iced_ui::divider::Divider;
use iced_ui::fab::Fab;
use iced_ui::icons::{self, Icon};
use iced_ui::list;
use iced_ui::screen::Screen;
use iced_ui::text::Text;
use iced_ui::top_app_bar::TopAppBar;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {}

#[derive(Default)]
pub(crate) struct OverviewPage;

impl super::PageView for OverviewPage {
    type Msg = Msg;
    const LABEL: &'static str = "Overview";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        // Top app bar
        let nav_icon: Element<'_, Msg> = icons::icon(Icon::Menu).into();
        let action: Element<'_, Msg> = icons::icon(Icon::CircleHelp).size(16).into();
        let app_bar = TopAppBar::new("My App")
            .navigation_icon(nav_icon)
            .action(action);

        // A list with badges
        let inbox_item = Badge::count(text("Inbox").size(14), 3);
        let updates_item = Badge::dot(text("Updates").size(14));
        let items_list = list::List::new()
            .push(list::Item::new(inbox_item))
            .push(list::Item::new(updates_item))
            .push(list::Item::new(text("Drafts").size(14)))
            .push(list::Item::new(text("Sent").size(14)));

        // A FAB
        let fab = Fab::new(icons::icon(Icon::Plus).size(24)).label(text("Compose").size(16));

        // Divider between sections
        let divider = Divider::horizontal();

        let screen = Screen::new(
            column![
                app_bar,
                divider,
                items_list,
                row![iced::widget::Space::new().width(Length::Fill), fab].padding(8),
            ]
            .spacing(16),
        );

        column![
            text("A composed layout demonstrating how iced_ui widgets work together.").size(14),
            Text::h2("Composed Layout"),
            screen,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
