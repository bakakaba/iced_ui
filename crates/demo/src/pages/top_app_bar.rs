use iced::widget::{column, text};
use iced_ui::icons::{self, Icon};
use iced_ui::text::Text;
use iced_ui::top_app_bar::TopAppBar;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {}

#[derive(Default)]
pub(crate) struct TopAppBarPage;

impl super::PageView for TopAppBarPage {
    type Msg = Msg;
    const LABEL: &'static str = "TopAppBar";
    const TITLE: &'static str = "Top App Bar";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let nav_icon: Element<'_, Msg> = icons::icon(Icon::ArrowLeft).into();
        let action1: Element<'_, Msg> = icons::icon(Icon::Search).size(16).into();
        let action2: Element<'_, Msg> = icons::icon(Icon::CircleHelp).size(16).into();

        let app_bar = TopAppBar::new("Page Title")
            .navigation_icon(nav_icon)
            .action(action1)
            .action(action2);

        column![
            text("Title bar with navigation icon, title, and action icons.").size(14),
            Text::h2("Default"),
            app_bar,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
